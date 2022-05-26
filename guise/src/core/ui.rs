use alloc::vec::Vec;
use core::alloc::Allocator;
use core::mem;
use core::ops::{BitOr, BitOrAssign, Range};

use arrayvec::ArrayString;

use crate::core::draw_list::{Command, DrawList, Vertex};
use crate::core::font_atlas::{FontAtlas, UnicodeRangeFlags};
use crate::core::math::{Rect, Vec2};

const ROOT_IDX: usize = 0;
const OVERLAY_ROOT_IDX: usize = 1;

const VERTICAL_RESIZE_FLAGS: CtrlFlags =
    CtrlFlags::SHRINK_TO_FIT_INLINE_VERTICAL | CtrlFlags::RESIZE_TO_FIT_VERTICAL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Inputs(u32);

impl Inputs {
    pub const MB_LEFT: Self = Self(0x01);
    pub const MB_RIGHT: Self = Self(0x02);
    pub const MB_MIDDLE: Self = Self(0x04);
    pub const MB_4: Self = Self(0x08);
    pub const MB_5: Self = Self(0x10);
    pub const MB_6: Self = Self(0x20);
    pub const MB_7: Self = Self(0x40);

    pub const KB_TAB: Self = Self(0x80);
    pub const KB_LEFT_ARROW: Self = Self(0x100);
    pub const KB_RIGHT_ARROW: Self = Self(0x200);
    pub const KB_UP_ARROW: Self = Self(0x400);
    pub const KB_DOWN_ARROW: Self = Self(0x800);
    pub const KB_PAGE_UP: Self = Self(0x1000);
    pub const KB_PAGE_DOWN: Self = Self(0x2000);
    pub const KB_HOME: Self = Self(0x4000);
    pub const KB_END: Self = Self(0x8000);
    pub const KB_INSERT: Self = Self(0x10000);
    pub const KB_DELETE: Self = Self(0x20000);
    pub const KB_BACKSPACE: Self = Self(0x40000);
    pub const KB_ENTER: Self = Self(0x80000);
    pub const KB_ESCAPE: Self = Self(0x100000);

    // TODO(yan): Fill in gamepad thingies.

    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self::MB_LEFT
        | Self::MB_RIGHT
        | Self::MB_MIDDLE
        | Self::MB_4
        | Self::MB_5
        | Self::MB_6
        | Self::MB_7
        | Self::KB_TAB
        | Self::KB_LEFT_ARROW
        | Self::KB_RIGHT_ARROW
        | Self::KB_UP_ARROW
        | Self::KB_DOWN_ARROW
        | Self::KB_PAGE_UP
        | Self::KB_PAGE_DOWN
        | Self::KB_HOME
        | Self::KB_END
        | Self::KB_INSERT
        | Self::KB_DELETE
        | Self::KB_BACKSPACE
        | Self::KB_ENTER
        | Self::KB_ESCAPE;

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn from_bits_truncate(bits: u32) -> Self {
        Self(Self::ALL.0 & bits)
    }

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl const BitOr for Inputs {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for Inputs {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layout {
    Free,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wrap {
    Word,
    Letter,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DrawPrimitive {
    Rect {
        rect: Rect,
        texture_rect: Rect,
        texture_id: u64,
        color: u32,
    },
    // TODO(yan): Circles, Rounded arcs, whatever..
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CtrlFlags(u32);

impl CtrlFlags {
    /// Whether the control should be affected by user interaction generated
    /// scrolling (and therefore capture scroll events when there is someplace
    /// to scroll). Regardless of this flag, controls can be scrolled
    /// programmatically.
    pub const CAPTURE_SCROLL: Self = Self(0x01);

    /// Whether the control should report being hovered (and therefore capture
    /// hover events). Hovering is tracked internally regardless of this flag,
    /// but not setting the flag let's the hover event flow to parent controls.
    pub const CAPTURE_HOVER: Self = Self(0x02);

    /// Whether the control should become active if one of its children becomes
    /// inactive. Controls can become active programmatically regardless of this
    /// flag.
    pub const CAPTURE_ACTIVE: Self = Self(0x04);

    #[allow(dead_code)]
    const __RESERVED: Self = Self(0x08);

    /// Whether to attempt shrinking the control's rect width to the width of
    /// its inline contents (text or geometry) before layout and render. This
    /// will only ever shrink and never grow.
    ///
    /// One usecase is creating controls that take up space based on their
    /// contents. They can start with rects occupying all available width (the
    /// parent's inner height) and use this flag to shrink their rect
    /// dynamically.
    ///
    /// This works, becuase the layout of inline content is immediately
    /// available when a control is popped.
    pub const SHRINK_TO_FIT_INLINE_HORIZONTAL: Self = Self(0x10);

    /// Whether to attempt shrinking the control's rect height to the height of
    /// its inline contents (text or geometry) before layout and render. This
    /// will only ever shrink and never grow.
    ///
    /// One usecase is creating controls that take up space based on their
    /// contents. They can start with rects occupying all available height (the
    /// parent's inner height) and use this flag to shrink their rect
    /// dynamically.
    ///
    /// This works, becuase the layout of inline content is immediately
    /// available when a control is popped.
    pub const SHRINK_TO_FIT_INLINE_VERTICAL: Self = Self(0x20);

    /// Whether to resize the control's rect width to the width of its contents,
    /// child or inline.
    ///
    /// One usecase is auto-sizing tooltips based on content.
    ///
    /// This has no downsides for non-interactive controls, because the layout
    /// pass computes the size of all of control's contents before they are used
    /// for rendering. Any interactivity may experience a one frame lag,
    /// however, because building the UI happens before layout is computed, and
    /// only has layout data from last frame, if any.
    pub const RESIZE_TO_FIT_HORIZONTAL: Self = Self(0x40);

    /// Whether to resize the control's rect height to the height of its contents,
    /// child or inline.
    ///
    /// One usecase is auto-sizing tooltips based on content.
    ///
    /// This has no downsides for non-interactive controls, because the layout
    /// pass computes the size of all of control's contents before they are used
    /// for rendering. Any interactivity may experience a one frame lag,
    /// however, because building the UI happens before layout is computed, and
    /// only has layout data from last frame, if any.
    pub const RESIZE_TO_FIT_VERTICAL: Self = Self(0x80);

    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self::CAPTURE_SCROLL
        | Self::CAPTURE_HOVER
        | Self::CAPTURE_ACTIVE
        | Self::SHRINK_TO_FIT_INLINE_HORIZONTAL
        | Self::SHRINK_TO_FIT_INLINE_VERTICAL
        | Self::RESIZE_TO_FIT_HORIZONTAL
        | Self::RESIZE_TO_FIT_VERTICAL;

    pub const ALL_SHRINK_TO_FIT_INLINE: Self =
        Self::SHRINK_TO_FIT_INLINE_HORIZONTAL | Self::SHRINK_TO_FIT_INLINE_VERTICAL;
    pub const ALL_RESIZE_TO_FIT: Self =
        Self::RESIZE_TO_FIT_HORIZONTAL | Self::RESIZE_TO_FIT_VERTICAL;

    pub fn bits(self) -> u32 {
        self.0
    }

    pub fn from_bits_truncate(bits: u32) -> Self {
        Self(Self::ALL.0 & bits)
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl const BitOr for CtrlFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for CtrlFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

pub type CtrlState = [u8; 64];

#[derive(Debug, Clone, PartialEq)]
struct CtrlNode {
    // Unique across siblings, but no further.
    id: u32,

    // TODO(yan): @Speed @Memory Make indices more compact. Option<usize> is 16
    // bytes, but we could carve out a niche.
    parent_idx: Option<usize>,
    child_idx: Option<usize>,
    sibling_idx: Option<usize>,

    // Deallocate if not current.
    last_frame: u32,
    // Used to sort free layout controls for detecting hover and rendering.
    last_frame_in_active_path: u32,

    // Layout things
    flags: CtrlFlags,
    layout: Layout,
    rect: Rect,
    padding: f32,
    border: f32,
    margin: f32,

    inline_content_rect: Option<Rect>,

    scroll_offset: Vec2,

    // TODO(yan): @Speed @Memory Isn't this a lot of state memory? Can we make
    // the number lower? Or only allocate state memory for controls that need
    // it?
    state: CtrlState,

    draw_self: bool,
    draw_self_border_color: u32,
    draw_self_background_color: u32,
    draw_range: Range<usize>,

    layout_cache_absolute_position: Vec2,
    layout_cache_content_size: Vec2,
}

pub struct Ui<A: Allocator + Clone> {
    // TODO(yan): @Memory We use this allocator for both permanent and temporary
    // memory, which requires some acrobatics to ensure we don't prevent
    // temporary memory reclamation, if the allocator is a bump
    // allocator. Should we split off a temporary allocator from this one for
    // internal use?
    allocator: A,

    draw_primitives: Vec<DrawPrimitive, A>,
    draw_list: DrawList<A>,

    font_atlas: FontAtlas<A>,
    font_atlas_texture_id: u64,

    tree: Vec<CtrlNode, A>,

    building_overlay: bool,
    build_parent_idx: Option<usize>,
    build_sibling_idx: Option<usize>,
    overlay_build_parent_idx: Option<usize>,
    overlay_build_sibling_idx: Option<usize>,

    current_frame: u32,

    window_size: Vec2,
    scroll_delta: Vec2,
    cursor_position: Vec2,
    inputs_pressed: Inputs,
    inputs_released: Inputs,
    received_characters: ArrayString<32>,

    active_ctrl_idx: Option<usize>,
    hovered_ctrl_idx: Option<usize>,
    hovered_capturing_ctrl_idx: Option<usize>,

    // TODO(yan): When exactly should we be capturing keyboard and mouse
    // automatically, when no control requests it? Currently we capture mouse
    // when something is hovered (ImGui does the same). ImGui also automatically
    // captures keyboard not just when a text field is active, but e.g. when
    // windows are being dragged around.
    want_capture_keyboard: bool,
    want_capture_mouse: bool,
}

impl<A: Allocator + Clone> Ui<A> {
    pub fn new_in(
        window_width: f32,
        window_height: f32,
        font_bytes: &[u8],
        font_unicode_range_flags: UnicodeRangeFlags,
        font_size: f32,
        // Maximum window scale factor the rasterizer is prepared for. If
        // displaying on a single monitor, that display's scale factor should be
        // used. If there are multiple monitors with different scales, pick
        // highest for sharpest looking fonts, or lower, if memory or speed is
        // an issue.
        font_rasterization_scale_factor: f32,
        allocator: A,
    ) -> Self {
        const NODE_CAPACITY: usize = 1024;

        let a1 = allocator.clone();
        let a2 = allocator.clone();
        let a3 = allocator.clone();
        let a4 = allocator.clone();

        let window_size = Vec2::new(window_width, window_height);
        let font_atlas = FontAtlas::new_in(
            font_bytes,
            font_unicode_range_flags,
            font_size,
            font_rasterization_scale_factor,
            a1,
        );

        let root_ctrl = CtrlNode {
            id: 0,

            parent_idx: None,
            child_idx: None,
            sibling_idx: None,

            last_frame: 0,
            last_frame_in_active_path: 0,

            flags: CtrlFlags::NONE,
            layout: Layout::Free,
            rect: Rect::from_points(Vec2::ZERO, window_size),
            padding: 0.0,
            border: 0.0,
            margin: 0.0,

            inline_content_rect: None,

            scroll_offset: Vec2::ZERO,

            state: [0; 64],

            draw_self: false,
            draw_self_border_color: 0,
            draw_self_background_color: 0,
            draw_range: 0..0,

            layout_cache_absolute_position: Vec2::ZERO,
            layout_cache_content_size: Vec2::ZERO,
        };

        let mut tree = Vec::with_capacity_in(NODE_CAPACITY, a2);
        tree.push(root_ctrl.clone());
        tree.push(root_ctrl);

        Self {
            allocator,

            draw_primitives: Vec::with_capacity_in(NODE_CAPACITY, a3),
            draw_list: DrawList::with_capacity_in(NODE_CAPACITY, a4),

            font_atlas,
            font_atlas_texture_id: 0,

            tree,

            building_overlay: false,
            build_parent_idx: None,
            build_sibling_idx: None,
            overlay_build_parent_idx: None,
            overlay_build_sibling_idx: None,

            current_frame: 0,

            window_size,
            scroll_delta: Vec2::ZERO,
            cursor_position: Vec2::ZERO,
            inputs_pressed: Inputs::empty(),
            inputs_released: Inputs::empty(),
            received_characters: ArrayString::new(),

            active_ctrl_idx: None,
            hovered_ctrl_idx: None,
            hovered_capturing_ctrl_idx: None,

            want_capture_keyboard: false,
            want_capture_mouse: false,
        }
    }

    pub fn set_font_atlas_texture_id(&mut self, font_atlas_texture_id: u64) {
        self.font_atlas_texture_id = font_atlas_texture_id;
    }

    pub fn set_window_size(&mut self, window_width: f32, window_height: f32) {
        self.window_size = Vec2::new(window_width, window_height);
    }

    pub fn scroll(&mut self, delta_x: f32, delta_y: f32) {
        self.scroll_delta += Vec2::new(delta_x, delta_y);
    }

    pub fn set_cursor_position(&mut self, cursor_x: f32, cursor_y: f32) {
        self.cursor_position = Vec2::new(cursor_x, cursor_y);
    }

    pub fn press_inputs(&mut self, inputs: Inputs) {
        self.inputs_pressed |= inputs;
    }

    pub fn release_inputs(&mut self, inputs: Inputs) {
        self.inputs_released |= inputs;
    }

    pub fn send_character(&mut self, character: char) {
        let _ = self.received_characters.try_push(character);
    }

    pub fn font_atlas_image_size(&self) -> (u16, u16) {
        self.font_atlas.image_size()
    }

    pub fn font_atlas_image_rgba8_unorm(&self) -> &[u8] {
        self.font_atlas.image_rgba8_unorm()
    }

    pub fn ctrl_count(&self) -> usize {
        self.tree.len()
    }

    pub fn want_capture_keyboard(&self) -> bool {
        self.want_capture_keyboard
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.want_capture_mouse
    }

    pub fn draw_list(&self) -> (&[Command], &[Vertex], &[u32]) {
        (
            self.draw_list.commands(),
            self.draw_list.vertices(),
            self.draw_list.indices(),
        )
    }

    pub fn begin_frame(&mut self) -> Frame<'_, A> {
        self.draw_primitives.clear();
        self.draw_list.clear();
        self.want_capture_keyboard = false;
        self.want_capture_mouse = false;

        self.current_frame = self.current_frame.wrapping_add(1);

        let root_ctrl = &mut self.tree[ROOT_IDX];
        root_ctrl.last_frame = self.current_frame;
        root_ctrl.last_frame_in_active_path = self.current_frame;
        root_ctrl.rect = Rect::from_points(Vec2::ZERO, self.window_size);

        let overlay_root_ctrl = &mut self.tree[OVERLAY_ROOT_IDX];
        overlay_root_ctrl.last_frame = self.current_frame;
        overlay_root_ctrl.last_frame_in_active_path = self.current_frame;
        overlay_root_ctrl.rect = Rect::from_points(Vec2::ZERO, self.window_size);

        //
        // Find hovered control.
        //
        // Look at the tree starting from the root and follow branches where the
        // child control's rect contains the cursor. First look at the overlay
        // tree, only then look at the base layer, if we didn't find a
        // hover-capturing ctrl.
        //
        // TODO(yan): Audit this. Not sure why we look for hovered node in the
        // base layer if we don't find hover-capturing node in the overlay.
        //
        self.hovered_capturing_ctrl_idx = None;
        self.hovered_ctrl_idx = find_hovered_ctrl(
            &self.tree,
            OVERLAY_ROOT_IDX,
            self.cursor_position,
            &self.allocator,
        );

        if let Some(hovered_ctrl_idx) = self.hovered_ctrl_idx {
            let mut ctrl_idx = hovered_ctrl_idx;
            let mut ctrl = &self.tree[hovered_ctrl_idx];

            while !ctrl.flags.intersects(CtrlFlags::CAPTURE_HOVER) && ctrl.parent_idx.is_some() {
                let parent_idx = ctrl.parent_idx.unwrap();

                ctrl_idx = parent_idx;
                ctrl = &self.tree[parent_idx];
            }

            if ctrl.flags.intersects(CtrlFlags::CAPTURE_HOVER) {
                self.hovered_capturing_ctrl_idx = Some(ctrl_idx);
                self.want_capture_mouse = true;
            }
        }

        if self.hovered_capturing_ctrl_idx == None {
            self.hovered_ctrl_idx =
                find_hovered_ctrl(&self.tree, ROOT_IDX, self.cursor_position, &self.allocator);
        }

        if let Some(hovered_ctrl_idx) = self.hovered_ctrl_idx {
            let mut ctrl_idx = hovered_ctrl_idx;
            let mut ctrl = &self.tree[hovered_ctrl_idx];

            while !ctrl.flags.intersects(CtrlFlags::CAPTURE_HOVER) && ctrl.parent_idx.is_some() {
                let parent_idx = ctrl.parent_idx.unwrap();

                ctrl_idx = parent_idx;
                ctrl = &self.tree[parent_idx];
            }

            if ctrl.flags.intersects(CtrlFlags::CAPTURE_HOVER) {
                self.hovered_capturing_ctrl_idx = Some(ctrl_idx);
                self.want_capture_mouse = true;
            }
        }

        fn find_hovered_ctrl<T: Allocator>(
            tree: &[CtrlNode],
            ctrl_idx: usize,
            cursor_position: Vec2,
            temp_allocator: &T,
        ) -> Option<usize> {
            let ctrl = &tree[ctrl_idx];
            let ctrl_rect_absolute = Rect::new(
                ctrl.layout_cache_absolute_position.x,
                ctrl.layout_cache_absolute_position.y,
                ctrl.rect.width,
                ctrl.rect.height,
            );

            if ctrl_rect_absolute.contains_point(cursor_position) {
                if ctrl.layout == Layout::Free {
                    // For free layout, we'd like to preserve the render order
                    // of controls when determining hover. The most recently
                    // active control (on top) has priority when determining
                    // hover, followed by the next most recently active control,
                    // all the way up to the least recently active control.

                    let mut siblings: Vec<(usize, u32), _> = Vec::new_in(temp_allocator);
                    if let Some(child_idx) = ctrl.child_idx {
                        let mut child = &tree[child_idx];
                        siblings.push((child_idx, child.last_frame_in_active_path));

                        while let Some(sibling_idx) = child.sibling_idx {
                            child = &tree[sibling_idx];
                            siblings.push((sibling_idx, child.last_frame_in_active_path));
                        }
                    }

                    siblings.sort_unstable_by_key(|&(_, frame)| frame);

                    for (sibling_idx, _) in siblings.into_iter().rev() {
                        if let Some(hovered_ctrl) =
                            find_hovered_ctrl(tree, sibling_idx, cursor_position, temp_allocator)
                        {
                            // This control is hovered, but also one of its
                            // children is.
                            return Some(hovered_ctrl);
                        }
                    }

                    // This control is hovered, but none of its children are.
                    Some(ctrl_idx)
                } else if let Some(child_idx) = ctrl.child_idx {
                    if let Some(hovered_ctrl) =
                        find_hovered_ctrl(tree, child_idx, cursor_position, temp_allocator)
                    {
                        // This control is hovered, but also one of its
                        // children is.
                        return Some(hovered_ctrl);
                    }

                    let mut child = &tree[child_idx];
                    while let Some(sibling_idx) = child.sibling_idx {
                        child = &tree[sibling_idx];

                        if let Some(hovered_ctrl) =
                            find_hovered_ctrl(tree, sibling_idx, cursor_position, temp_allocator)
                        {
                            // This control is hovered, but also one of its
                            // children is.
                            return Some(hovered_ctrl);
                        }
                    }

                    // This control is hovered, but none of its children are.
                    Some(ctrl_idx)
                } else {
                    // This control is hovered and has no children to explore.
                    Some(ctrl_idx)
                }
            } else {
                // This control is not hovered.
                None
            }
        }

        //
        // Scroll a control.
        //
        // If the hovered control doesn't want scrolling or doesn't have
        // overflow it could scroll, walk the tree up to the first eligible
        // control and scroll that!
        //
        if self.scroll_delta != Vec2::ZERO {
            if let Some(idx) = self.hovered_ctrl_idx {
                let mut ctrl = &mut self.tree[idx];
                let mut ctrl_scroll_size = Vec2::ZERO.max(
                    ctrl.layout_cache_content_size - ctrl.rect.size()
                        + 2.0 * ctrl.padding
                        + 2.0 * ctrl.border,
                );
                let mut ctrl_scroll_offset_new =
                    (ctrl.scroll_offset - self.scroll_delta).clamp(Vec2::ZERO, ctrl_scroll_size);
                let mut ctrl_can_scroll = ctrl.flags.intersects(CtrlFlags::CAPTURE_SCROLL)
                    && ctrl_scroll_offset_new != ctrl.scroll_offset;

                while !ctrl_can_scroll && ctrl.parent_idx.is_some() {
                    let parent_idx = ctrl.parent_idx.unwrap();

                    ctrl = &mut self.tree[parent_idx];
                    ctrl_scroll_size = Vec2::ZERO.max(
                        ctrl.layout_cache_content_size - ctrl.rect.size()
                            + 2.0 * ctrl.padding
                            + 2.0 * ctrl.border,
                    );
                    ctrl_scroll_offset_new = (ctrl.scroll_offset - self.scroll_delta)
                        .clamp(Vec2::ZERO, ctrl_scroll_size);
                    ctrl_can_scroll = ctrl.flags.intersects(CtrlFlags::CAPTURE_SCROLL)
                        && ctrl_scroll_offset_new != ctrl.scroll_offset;
                }

                if ctrl_can_scroll {
                    ctrl.scroll_offset = ctrl_scroll_offset_new;
                }
            }
        }

        self.build_parent_idx = Some(ROOT_IDX);
        self.build_sibling_idx = None;
        self.overlay_build_parent_idx = Some(OVERLAY_ROOT_IDX);
        self.overlay_build_sibling_idx = None;

        Frame { ui: self }
    }

    pub fn end_frame(&mut self) {
        assert!(
            self.build_parent_idx == Some(ROOT_IDX),
            "Is there a pop_ctrl for every push_ctrl?",
        );
        assert!(
            self.overlay_build_parent_idx == Some(OVERLAY_ROOT_IDX),
            "Is there a pop_ctrl for every push_ctrl?",
        );

        // Perform cleanup on the roots analogous to the cleanup that happens in
        // pop_ctrl for other (not root) controls.
        {
            // NB: build_parent_idx and overlay_build_parent_idx assertions
            // already happen above.
            debug_assert!(self.tree[ROOT_IDX].sibling_idx == None);
            debug_assert!(self.tree[OVERLAY_ROOT_IDX].sibling_idx == None);

            if let Some(build_sibling_idx) = self.build_sibling_idx {
                self.tree[build_sibling_idx].sibling_idx = None;
            } else {
                self.tree[self.build_parent_idx.unwrap()].child_idx = None;
            }

            if let Some(overlay_build_sibling_idx) = self.overlay_build_sibling_idx {
                self.tree[overlay_build_sibling_idx].sibling_idx = None;
            } else {
                self.tree[self.overlay_build_parent_idx.unwrap()].child_idx = None;
            }
        }

        // Discover reachachable dead controls in the tree. If there are any, we
        // did something wrong. There can be dead nodes, but they must not be
        // reachable.
        #[cfg(debug_assertions)]
        {
            dead_discovery(&self.tree, ROOT_IDX, self.current_frame);
            dead_discovery(&self.tree, OVERLAY_ROOT_IDX, self.current_frame);

            fn dead_discovery(tree: &[CtrlNode], ctrl_idx: usize, current_frame: u32) {
                let mut ctrl = &tree[ctrl_idx];

                if ctrl.last_frame != current_frame {
                    let id = ctrl.id;
                    panic!("Reachable dead control found at {ctrl_idx}, id: {id}");
                }

                if let Some(child_idx) = ctrl.child_idx {
                    dead_discovery(tree, child_idx, current_frame);

                    while let Some(sibling_idx) = ctrl.sibling_idx {
                        dead_discovery(tree, sibling_idx, current_frame);
                        ctrl = &tree[sibling_idx];
                    }
                }
            }
        }

        //
        // Collect dead controls.
        //
        // Go over every control and see if it's dead. If it is, swap_remove
        // it. This invalidates all indices pointing to the last, possibly live
        // control, so we record the fact that a relocation happened and later
        // fix up the references.
        //
        // By this point, dead controls should not be referenced by any live
        // control, so there is no need to fix references to them.
        //
        // TODO(yan): @Speed This GC sucks at maintaining locality between
        // siblings. Do some kind of double-buffering and compaction.
        //

        let mut relocations: Vec<(usize, usize), _> =
            Vec::with_capacity_in(self.tree.len(), &self.allocator);

        fn apply_relocation(idx_to_relocate: &mut Option<usize>, src: usize, dst: usize) {
            if let Some(idx) = idx_to_relocate.as_mut() {
                if *idx == src {
                    *idx = dst;
                }
            }
        }

        let mut ctrl_idx = 0;
        while ctrl_idx < self.tree.len() {
            if self.tree[ctrl_idx].last_frame != self.current_frame {
                // The swapped in control could be dead too. Keep doing
                // swap_remove until we find a live control, only then record
                // the relocation.
                while ctrl_idx < self.tree.len()
                    && self.tree[ctrl_idx].last_frame != self.current_frame
                {
                    self.tree.swap_remove(ctrl_idx);
                }

                // Only record the relocation if we found a live control - the
                // previous loop either stopped at the end of the tree vec, or
                // by finding a live control.
                if ctrl_idx < self.tree.len() {
                    relocations.push((self.tree.len(), ctrl_idx));
                }
            }

            ctrl_idx += 1;
        }

        // Apply relocations.
        for &(src, dst) in &relocations {
            apply_relocation(&mut self.active_ctrl_idx, src, dst);

            for ctrl in &mut self.tree {
                apply_relocation(&mut ctrl.parent_idx, src, dst);
                apply_relocation(&mut ctrl.child_idx, src, dst);
                apply_relocation(&mut ctrl.sibling_idx, src, dst);
            }
        }

        // NB: Drop relocations eagerly, so that if the allocator is a bump
        // allocator, we don't prevent it from reclaiming the memory.
        drop(relocations);

        //
        // Update layout.
        //
        // Because the build phase is now done, we can incorporate all the
        // layout changes. They will be used for this frame's render phase, and
        // next frame's build phase. We update both the base layer and the
        // overlay.
        //
        layout(&mut self.tree, ROOT_IDX, Vec2::ZERO);
        layout(&mut self.tree, OVERLAY_ROOT_IDX, Vec2::ZERO);

        fn layout(tree: &mut [CtrlNode], ctrl_idx: usize, ctrl_absolute_position_base: Vec2) {
            // TODO(yan): For horizontal and vertical layouts we advance the
            // position by the width and height of the rect of the current
            // control, but what if that control has its position offset by the
            // X or Y of the rect? (e.g. if X=100, should we advance the
            // horizontal cursor by an additional 100 pixels?)

            let ctrl = &tree[ctrl_idx];
            let ctrl_flags = ctrl.flags;
            let ctrl_layout = ctrl.layout;
            let ctrl_inline_content_rect = ctrl.inline_content_rect;
            let ctrl_absolute_position =
                ctrl_absolute_position_base + ctrl.rect.min_point() + ctrl.margin;

            if let Some(child_idx) = ctrl.child_idx {
                let child_absolute_position_base =
                    ctrl_absolute_position + ctrl.border + ctrl.padding - ctrl.scroll_offset;

                layout(tree, child_idx, child_absolute_position_base);

                let mut child = &tree[child_idx];
                let mut child_margin_rect = child.rect.offset(child.margin);
                let mut child_absolute_position_offset = match ctrl_layout {
                    Layout::Free => Vec2::ZERO,
                    Layout::Horizontal => Vec2::new(child_margin_rect.width, 0.0),
                    Layout::Vertical => Vec2::new(0.0, child_margin_rect.height),
                };

                let mut max_point = child_margin_rect.max_point();

                while let Some(sibling_idx) = child.sibling_idx {
                    layout(
                        tree,
                        sibling_idx,
                        child_absolute_position_base + child_absolute_position_offset,
                    );

                    child = &tree[sibling_idx];
                    child_margin_rect = child.rect.offset(child.margin);

                    match ctrl_layout {
                        Layout::Free => {
                            max_point = max_point.max(child_margin_rect.max_point());
                        }
                        Layout::Horizontal => {
                            child_absolute_position_offset += Vec2::X * child_margin_rect.width;
                            max_point.x += child_margin_rect.width;
                            max_point.y = max_point.y.max(child_margin_rect.max_y());
                        }
                        Layout::Vertical => {
                            child_absolute_position_offset += Vec2::Y * child_margin_rect.height;
                            max_point.x = max_point.x.max(child_margin_rect.max_x());
                            max_point.y += child_margin_rect.height;
                        }
                    }
                }

                if let Some(inline_content_rect) = ctrl_inline_content_rect {
                    max_point = max_point.max(inline_content_rect.max_point());
                }

                let ctrl_mut = &mut tree[ctrl_idx];
                ctrl_mut.layout_cache_absolute_position = ctrl_absolute_position;
                ctrl_mut.layout_cache_content_size = max_point;
            } else {
                let ctrl_mut = &mut tree[ctrl_idx];

                ctrl_mut.layout_cache_absolute_position = ctrl_absolute_position;
                if let Some(inline_content_rect) = ctrl_inline_content_rect {
                    ctrl_mut.layout_cache_content_size = inline_content_rect.max_point();
                } else {
                    ctrl_mut.layout_cache_content_size = Vec2::ZERO;
                }
            }

            if ctrl_flags.intersects(CtrlFlags::ALL_RESIZE_TO_FIT) {
                let ctrl_mut = &mut tree[ctrl_idx];

                let offset = 2.0 * ctrl_mut.border + 2.0 * ctrl_mut.padding;
                let x = ctrl_mut.rect.x;
                let y = ctrl_mut.rect.y;

                let width = if ctrl_flags.intersects(CtrlFlags::RESIZE_TO_FIT_HORIZONTAL) {
                    ctrl_mut.layout_cache_content_size.x + offset
                } else {
                    ctrl_mut.rect.width
                };

                let height = if ctrl_flags.intersects(CtrlFlags::RESIZE_TO_FIT_VERTICAL) {
                    ctrl_mut.layout_cache_content_size.y + offset
                } else {
                    ctrl_mut.rect.height
                };

                ctrl_mut.rect = Rect::new(x, y, width, height);
            }
        }

        //
        // Render into the draw lists. First the base, then the overlay.
        //
        render(
            &self.tree,
            ROOT_IDX,
            Rect::from_points(Vec2::ZERO, self.window_size),
            &self.draw_primitives,
            self.font_atlas_texture_id,
            &mut self.draw_list,
            &self.allocator,
        );
        render(
            &self.tree,
            OVERLAY_ROOT_IDX,
            Rect::from_points(Vec2::ZERO, self.window_size),
            &self.draw_primitives,
            self.font_atlas_texture_id,
            &mut self.draw_list,
            &self.allocator,
        );

        // TODO(yan): @Memory If the allocator is a bump allocator, we
        // potentially prevent it from reclaiming memory if draw_list grows.
        fn render<A: Allocator + Clone>(
            tree: &[CtrlNode],
            ctrl_idx: usize,
            parent_ctrl_scissor_rect: Rect,
            draw_primitives: &[DrawPrimitive],
            font_atlas_texture_id: u64,
            draw_list: &mut DrawList<A>,
            temp_allocator: &A,
        ) {
            let ctrl = &tree[ctrl_idx];
            let ctrl_rect_absolute = Rect::new(
                ctrl.layout_cache_absolute_position.x,
                ctrl.layout_cache_absolute_position.y,
                ctrl.rect.width,
                ctrl.rect.height,
            );

            let ctrl_scissor_rect = parent_ctrl_scissor_rect
                .clamp_rect(ctrl_rect_absolute)
                .inset(ctrl.border);

            // Some renderer backends dislike scissor rect with zero or negative
            // dimensions, as well as dimensions greater than the surface
            // dimensions. If we get dangerously close, let's not render
            // anything.
            if ctrl_scissor_rect.width < 1.0 || ctrl_scissor_rect.height < 1.0 {
                return;
            }

            if ctrl.draw_self {
                let border_color = ctrl.draw_self_border_color;
                let background_color = ctrl.draw_self_background_color;

                let ctrl_padding_rect_absolute = ctrl_rect_absolute.inset(ctrl.border);

                if !ctrl_rect_absolute.is_empty() && !ctrl_padding_rect_absolute.is_empty() {
                    // NB: f32::max is used in substractions here because fp
                    // precision commonly caused the result to be below 0, which
                    // is a big no-no for Rect::new.

                    let outer = ctrl_rect_absolute;
                    let inner = ctrl_padding_rect_absolute;

                    let lx = outer.x;
                    let ly = outer.y;
                    let lwidth = f32::max(0.0, inner.x - outer.x);
                    let lheight = outer.height;
                    let left = Rect::new(lx, ly, lwidth, lheight);

                    let tx = inner.x;
                    let ty = outer.y;
                    let twidth = inner.width;
                    let theight = f32::max(0.0, inner.y - outer.y);
                    let top = Rect::new(tx, ty, twidth, theight);

                    let rx = inner.x + inner.width;
                    let ry = outer.y;
                    let rwidth = f32::max(0.0, outer.width - inner.width - lwidth);
                    let rheight = outer.height;
                    let right = Rect::new(rx, ry, rwidth, rheight);

                    let bx = inner.x;
                    let by = inner.y + inner.height;
                    let bwidth = inner.width;
                    let bheight = f32::max(0.0, outer.height - inner.height - theight);
                    let bottom = Rect::new(bx, by, bwidth, bheight);

                    if !left.is_empty() {
                        draw_list.draw_rect(
                            left,
                            Rect::ZERO,
                            border_color,
                            parent_ctrl_scissor_rect,
                            font_atlas_texture_id,
                        );
                    }

                    if !top.is_empty() {
                        draw_list.draw_rect(
                            top,
                            Rect::ZERO,
                            border_color,
                            parent_ctrl_scissor_rect,
                            font_atlas_texture_id,
                        );
                    }

                    if !right.is_empty() {
                        draw_list.draw_rect(
                            right,
                            Rect::ZERO,
                            border_color,
                            parent_ctrl_scissor_rect,
                            font_atlas_texture_id,
                        );
                    }

                    if !bottom.is_empty() {
                        draw_list.draw_rect(
                            bottom,
                            Rect::ZERO,
                            border_color,
                            parent_ctrl_scissor_rect,
                            font_atlas_texture_id,
                        );
                    }
                }

                draw_list.draw_rect(
                    ctrl_padding_rect_absolute,
                    Rect::ZERO,
                    background_color,
                    parent_ctrl_scissor_rect,
                    font_atlas_texture_id,
                );
            }

            for draw_primitive_idx in ctrl.draw_range.clone() {
                let draw_primitive = &draw_primitives[draw_primitive_idx];
                match draw_primitive {
                    DrawPrimitive::Rect {
                        rect,
                        texture_rect,
                        texture_id,
                        color,
                    } => {
                        let rect = *rect + ctrl_rect_absolute.min_point() - ctrl.scroll_offset;
                        draw_list.draw_rect(
                            rect,
                            *texture_rect,
                            *color,
                            ctrl_scissor_rect,
                            *texture_id,
                        );
                    }
                }
            }

            if ctrl.layout == Layout::Free {
                // For free layout, we'd like to preserve render order of
                // controls, e.g. we render least recently active control first,
                // then a more recently active control, all the way up to the
                // currently active control. To that end, we sort the the
                // siblings by last frame in active path.
                let mut siblings: Vec<(usize, u32), _> = Vec::new_in(temp_allocator);
                if let Some(child_idx) = ctrl.child_idx {
                    let mut ctrl = &tree[child_idx];

                    siblings.push((child_idx, ctrl.last_frame_in_active_path));

                    while let Some(sibling_idx) = ctrl.sibling_idx {
                        ctrl = &tree[sibling_idx];
                        siblings.push((sibling_idx, ctrl.last_frame_in_active_path));
                    }
                }

                siblings.sort_unstable_by_key(|&(_, frame)| frame);

                for (sibling_idx, _) in siblings {
                    render(
                        tree,
                        sibling_idx,
                        ctrl_scissor_rect,
                        draw_primitives,
                        font_atlas_texture_id,
                        draw_list,
                        temp_allocator,
                    );
                }
            } else {
                // For horizontal and vertical layouts, we don't need any
                // sorting and just iterate over the controls in definition
                // order.
                if let Some(child_idx) = ctrl.child_idx {
                    render(
                        tree,
                        child_idx,
                        ctrl_scissor_rect,
                        draw_primitives,
                        font_atlas_texture_id,
                        draw_list,
                        temp_allocator,
                    );

                    let mut child = &tree[child_idx];
                    while let Some(sibling_idx) = child.sibling_idx {
                        child = &tree[sibling_idx];

                        render(
                            tree,
                            sibling_idx,
                            ctrl_scissor_rect,
                            draw_primitives,
                            font_atlas_texture_id,
                            draw_list,
                            temp_allocator,
                        );
                    }
                }
            }
        }

        self.build_parent_idx = None;
        self.build_sibling_idx = None;

        // NB: Clear inputs from platform to GUI.
        self.scroll_delta = Vec2::ZERO;
        self.inputs_pressed = Inputs::empty();
        self.inputs_released = Inputs::empty();
        self.received_characters.clear();
    }
}

pub struct Frame<'a, A: Allocator + Clone> {
    ui: &'a mut Ui<A>,
}

impl<'a, A: Allocator + Clone> Frame<'a, A> {
    pub fn push_ctrl(&mut self, id: u32) -> Ctrl<'_, A> {
        // Push a control onto the tree. The control can either be completely
        // new, or already present in the tree from previous frame. Controls are
        // identified by their ID, which has to be unique among children of a
        // single control.
        //
        // Whether inserting a new control or updating an existing one, we must
        // never unlink a control already present in the tree, because it may
        // yet be updated later this frame and we would have lost its state.
        //
        // Updated controls are first temporarily unlinked from the tree, and
        // then re-inserted at the current position. This means that dead
        // controls (if any) will be located after the live controls once the UI
        // is built.
        //
        // Current position in the tree is tracked by two indices:
        // build_parent_idx and build_sibling_idx, pointing to the current
        // parent and last inserted sibling, respectively.
        //
        // Note: Changing the control's layout options invalidates the layout
        // from last frame for this control and all its sibling successors, and
        // their children, but so does re-ordering, or not updating a control
        // from earlier frame. The layout will become valid on the next frame
        // once again.

        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let draw_range = {
            let next_idx = self.ui.draw_primitives.len();
            next_idx..next_idx
        };

        // TODO(yan): @Speed only search from build_sibling.sibling_idx, if
        // build_sibling already exists.
        let found_idx_and_prev_idx = {
            let parent = &self.ui.tree[build_parent_idx];

            // TODO(yan): @Speed This is quadratic. Not great.
            if let Some(child_idx) = parent.child_idx {
                let mut ctrl = &mut self.ui.tree[child_idx];

                if ctrl.id == id {
                    Some((child_idx, None))
                } else {
                    let mut result = None;

                    let mut ctrl_idx = child_idx;
                    while let Some(sibling_idx) = ctrl.sibling_idx {
                        let prev_ctrl_idx = ctrl_idx;
                        ctrl_idx = sibling_idx;
                        ctrl = &mut self.ui.tree[sibling_idx];

                        if ctrl.id == id {
                            result = Some((ctrl_idx, Some(prev_ctrl_idx)));
                            break;
                        }
                    }

                    result
                }
            } else {
                None
            }
        };

        let current_idx = if let Some((found_idx, found_prev_idx)) = found_idx_and_prev_idx {
            let ctrl = &mut self.ui.tree[found_idx];

            // We do not support re-entrancy. Controls can only be updated
            // once. This simplifies things:
            //
            // - We know that found_idx != build_sibling_idx, because the build
            //   sibling would have to be pushed and popped before,
            //
            // - We know that found_idx hasn't been pushed yet.
            //
            // TODO(yan): @Correctness This assert goes off if we render the
            // component only on some frames (discoverd by drawing a conditional
            // window in PH). We most definitely were not updating the same
            // component multiple times per frame, so this is an issue with
            // unlinking dead controls and/or GC?
            assert!(
                ctrl.last_frame != self.ui.current_frame,
                "Attempt to update the same control ({id}) twice in one frame",
            );

            ctrl.last_frame = self.ui.current_frame;
            ctrl.inline_content_rect = None;
            ctrl.draw_range = draw_range;

            // After updating the control's data, we unlink the control from its
            // original place and re-link as either the next sibling of the
            // build sibling (if build sibling already exists) or first child of
            // the build parent (if the build sibling doesn't exist yet).
            if let Some(found_prev_idx) = found_prev_idx {
                self.ui.tree[found_prev_idx].sibling_idx = ctrl.sibling_idx;
            }

            // Re-link the control as next sibling of the build sibling or
            // as first child of build parent (in case there is no build
            // sibling yet).
            if let Some(build_sibling_idx) = self.ui.build_sibling_idx {
                let build_sibling = &mut self.ui.tree[build_sibling_idx];
                let build_sibling_next_sibling_idx = build_sibling.sibling_idx;

                // If we are already positioned correctly, relinking would
                // create a cycle.
                if build_sibling_next_sibling_idx != Some(found_idx) {
                    build_sibling.sibling_idx = Some(found_idx);
                    self.ui.tree[found_idx].sibling_idx = build_sibling_next_sibling_idx;
                }
            } else {
                let build_parent = &mut self.ui.tree[build_parent_idx];
                let build_parent_child_idx = build_parent.child_idx;

                // If we are already positioned correctly, relinking would
                // create a cycle.
                if build_parent_child_idx != Some(found_idx) {
                    build_parent.child_idx = Some(found_idx);
                    self.ui.tree[found_idx].sibling_idx = build_parent_child_idx;
                }
            }

            found_idx
        } else {
            let idx = self.ui.tree.len();

            // Preserve links to controls from previous frame so that they can be
            // found by future calls to push_ctrl in this subtree and depth.
            let sibling_idx = if let Some(build_sibling_idx) = self.ui.build_sibling_idx {
                let build_sibling = &mut self.ui.tree[build_sibling_idx];
                let build_sibling_next_sibling_idx = build_sibling.sibling_idx;

                build_sibling.sibling_idx = Some(idx);
                build_sibling_next_sibling_idx
            } else {
                let build_parent = &mut self.ui.tree[build_parent_idx];
                let build_parent_child_idx = build_parent.child_idx;

                build_parent.child_idx = Some(idx);
                build_parent_child_idx
            };

            self.ui.tree.push(CtrlNode {
                id,

                parent_idx: Some(build_parent_idx),
                child_idx: None,
                sibling_idx,

                last_frame: self.ui.current_frame,
                last_frame_in_active_path: 0,

                flags: CtrlFlags::NONE,
                layout: Layout::Free,
                rect: Rect::ZERO,
                padding: 0.0,
                border: 0.0,
                margin: 0.0,

                inline_content_rect: None,

                scroll_offset: Vec2::ZERO,

                state: [0; 64],

                draw_self: false,
                draw_self_border_color: 0,
                draw_self_background_color: 0,
                draw_range,

                layout_cache_absolute_position: Vec2::ZERO,
                layout_cache_content_size: Vec2::ZERO,
            });

            idx
        };

        self.ui.build_parent_idx = Some(current_idx);
        self.ui.build_sibling_idx = None;

        Ctrl {
            idx: current_idx,
            ui: self.ui,
        }
    }

    pub fn pop_ctrl(&mut self) {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();

        // Finalize the parent and last inserted sibling controls and clean
        // their indices so that they only reference live controls. If no child
        // controls were inserted, clear the parent's child references (which
        // could contain dead controls from previous frame). Also cut off the
        // dead sibling controls of the last sibling here, so that they are not
        // reachable.

        // TODO(yan): @Correctness Assert that push_ctrl and pop_ctrl are
        // parenthesized correctly! Count current tree depth and assert
        // something in both pop_ctrl and end_frame?

        let build_parent = &mut self.ui.tree[build_parent_idx];
        let build_parent_parent_idx = build_parent.parent_idx;

        if build_parent
            .flags
            .intersects(CtrlFlags::ALL_SHRINK_TO_FIT_INLINE)
        {
            assert!(build_parent.child_idx == None);

            if let Some(inline_content_rect) = build_parent.inline_content_rect {
                let width = if build_parent
                    .flags
                    .intersects(CtrlFlags::SHRINK_TO_FIT_INLINE_HORIZONTAL)
                {
                    f32::min(
                        build_parent.rect.width,
                        inline_content_rect.x + inline_content_rect.width,
                    )
                } else {
                    build_parent.rect.width
                };

                let height = if build_parent
                    .flags
                    .intersects(CtrlFlags::SHRINK_TO_FIT_INLINE_VERTICAL)
                {
                    f32::min(
                        build_parent.rect.height,
                        inline_content_rect.y + inline_content_rect.height,
                    )
                } else {
                    build_parent.rect.height
                };

                build_parent.rect =
                    Rect::new(build_parent.rect.x, build_parent.rect.y, width, height);
            }
        }

        if let Some(build_sibling_idx) = self.ui.build_sibling_idx {
            self.ui.tree[build_sibling_idx].sibling_idx = None;
        } else {
            build_parent.child_idx = None;
        }

        self.ui.build_parent_idx = build_parent_parent_idx;
        self.ui.build_sibling_idx = Some(build_parent_idx);
    }

    pub fn begin_overlay(&mut self) {
        assert!(!self.ui.building_overlay);

        mem::swap(
            &mut self.ui.build_parent_idx,
            &mut self.ui.overlay_build_parent_idx,
        );
        mem::swap(
            &mut self.ui.build_sibling_idx,
            &mut self.ui.overlay_build_sibling_idx,
        );

        self.ui.building_overlay = true;
    }

    pub fn end_overlay(&mut self) {
        assert!(self.ui.building_overlay);

        mem::swap(
            &mut self.ui.build_parent_idx,
            &mut self.ui.overlay_build_parent_idx,
        );
        mem::swap(
            &mut self.ui.build_sibling_idx,
            &mut self.ui.overlay_build_sibling_idx,
        );

        self.ui.building_overlay = false;
    }

    pub fn font_atlas_texture_id(&self) -> u64 {
        self.ui.font_atlas_texture_id
    }

    pub fn window_size(&self) -> Vec2 {
        self.ui.window_size
    }

    pub fn cursor_position(&self) -> Vec2 {
        self.ui.cursor_position
    }

    pub fn inputs_pressed(&self) -> Inputs {
        self.ui.inputs_pressed
    }

    pub fn inputs_released(&self) -> Inputs {
        self.ui.inputs_released
    }

    pub fn received_characters(&self) -> &str {
        &self.ui.received_characters
    }

    pub fn ctrl_state(&self) -> &CtrlState {
        &self.ui.tree[self.ui.build_parent_idx.unwrap()].state
    }

    pub fn ctrl_state_mut(&mut self) -> &mut CtrlState {
        &mut self.ui.tree[self.ui.build_parent_idx.unwrap()].state
    }

    pub fn ctrl_absolute_position(&self) -> Vec2 {
        self.ui.tree[self.ui.build_parent_idx.unwrap()].layout_cache_absolute_position
    }

    pub fn ctrl_inner_size(&self) -> Vec2 {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let parent = &self.ui.tree[build_parent_idx];
        let rect = parent.rect.inset(parent.border + parent.padding);

        rect.size()
    }

    pub fn ctrl_count(&self) -> usize {
        self.ui.ctrl_count()
    }
}

pub struct Ctrl<'a, A: Allocator + Clone> {
    idx: usize,
    ui: &'a mut Ui<A>,
}

// TODO(yan): Vertical and horizontal align.
impl<'a, A: Allocator + Clone> Ctrl<'a, A> {
    pub fn set_active(&mut self, active: bool) {
        if active {
            self.ui.active_ctrl_idx = Some(self.idx);

            let mut ctrl = &mut self.ui.tree[self.idx];
            ctrl.last_frame_in_active_path = self.ui.current_frame;

            while let Some(ctrl_idx) = ctrl.parent_idx {
                ctrl = &mut self.ui.tree[ctrl_idx];
                ctrl.last_frame_in_active_path = self.ui.current_frame;
            }
        } else if let Some(active_ctrl_idx) = self.ui.active_ctrl_idx {
            if active_ctrl_idx == self.idx {
                // If this was the active control, it relinquishes the active
                // status the the first control up the tree that wants to
                // capture it. When that happens, the capturing control and all
                // its parents get their last_frame_in_active_path updated.

                let current_ctrl = &self.ui.tree[self.idx];

                if let Some(parent_idx) = current_ctrl.parent_idx {
                    let mut ctrl_idx = parent_idx;
                    let mut ctrl = &mut self.ui.tree[parent_idx];

                    while !ctrl.flags.intersects(CtrlFlags::CAPTURE_ACTIVE)
                        && ctrl.parent_idx.is_some()
                    {
                        ctrl_idx = ctrl.parent_idx.unwrap();
                        ctrl = &mut self.ui.tree[ctrl_idx];
                    }

                    if ctrl.flags.intersects(CtrlFlags::CAPTURE_ACTIVE) {
                        self.ui.active_ctrl_idx = Some(ctrl_idx);

                        ctrl.last_frame_in_active_path = self.ui.current_frame;

                        while let Some(ctrl_idx) = ctrl.parent_idx {
                            ctrl = &mut self.ui.tree[ctrl_idx];
                            ctrl.last_frame_in_active_path = self.ui.current_frame
                        }
                    } else {
                        self.ui.active_ctrl_idx = None;
                    }
                }
            }
        }
    }

    pub fn set_flags(&mut self, flags: CtrlFlags) {
        self.ui.tree[self.idx].flags = flags;
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.ui.tree[self.idx].layout = layout;
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.ui.tree[self.idx].rect = rect;
    }

    pub fn set_padding(&mut self, padding: f32) {
        self.ui.tree[self.idx].padding = padding;
    }

    pub fn set_border(&mut self, border: f32) {
        self.ui.tree[self.idx].border = border;
    }

    pub fn set_margin(&mut self, margin: f32) {
        self.ui.tree[self.idx].margin = margin;
    }

    pub fn set_scroll_offset_x(&mut self, scroll_offset: f32) {
        self.ui.tree[self.idx].scroll_offset.x = scroll_offset;
    }

    pub fn set_scroll_offset_y(&mut self, scroll_offset: f32) {
        self.ui.tree[self.idx].scroll_offset.y = scroll_offset;
    }

    pub fn set_draw_self(&mut self, draw_self: bool) {
        self.ui.tree[self.idx].draw_self = draw_self;
    }

    pub fn set_draw_self_border_color(&mut self, border_color: u32) {
        self.ui.tree[self.idx].draw_self_border_color = border_color;
    }

    pub fn set_draw_self_background_color(&mut self, background_color: u32) {
        self.ui.tree[self.idx].draw_self_background_color = background_color;
    }

    pub fn hovered(&self) -> bool {
        self.ui.build_parent_idx == self.ui.hovered_capturing_ctrl_idx
    }

    pub fn active(&self) -> bool {
        self.ui.active_ctrl_idx == Some(self.idx)
    }

    pub fn state(&self) -> &CtrlState {
        &self.ui.tree[self.idx].state
    }

    pub fn state_mut(&mut self) -> &mut CtrlState {
        &mut self.ui.tree[self.idx].state
    }

    pub fn absolute_position(&self) -> Vec2 {
        self.ui.tree[self.idx].layout_cache_absolute_position
    }

    pub fn inner_size(&self) -> Vec2 {
        let ctrl = &self.ui.tree[self.idx];
        let rect = ctrl.rect.inset(ctrl.border + ctrl.padding);

        rect.size()
    }

    pub fn scroll_offset_x(&self) -> f32 {
        self.ui.tree[self.idx].scroll_offset.x
    }

    pub fn scroll_offset_y(&self) -> f32 {
        self.ui.tree[self.idx].scroll_offset.y
    }

    pub fn request_want_capture_keyboard(&mut self) {
        self.ui.want_capture_keyboard = true;
    }

    pub fn request_want_capture_mouse(&mut self) {
        self.ui.want_capture_mouse = true;
    }

    pub fn draw_rect(
        &mut self,
        include_in_inline_content_rect: bool,
        rect: Rect,
        texture_rect: Rect,
        color: u32,
        texture_id: u64,
    ) {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let next_draw_primitive_idx = self.ui.draw_primitives.len();

        let parent = &mut self.ui.tree[build_parent_idx];
        assert!(parent.draw_range.end == next_draw_primitive_idx);

        self.ui.draw_primitives.push(DrawPrimitive::Rect {
            rect,
            texture_rect,
            texture_id,
            color,
        });

        parent.draw_range.end += 1;
        if include_in_inline_content_rect {
            if let Some(inline_content_rect) = &mut parent.inline_content_rect {
                *inline_content_rect = inline_content_rect.extend_by_rect(rect);
            } else {
                parent.inline_content_rect = Some(rect);
            }
        }
    }

    pub fn draw_text(
        &mut self,
        include_in_inline_content_rect: bool,
        available_rect: Option<Rect>,
        inset_amount: f32,
        text: &str,
        horizontal_align: Align,
        vertical_align: Align,
        wrap: Wrap,
        color: u32,
    ) {
        assert!(inset_amount >= 0.0);

        // TODO(yan): This has layout issues (characters not being aligned
        // vertically to the baseline) on Roboto, IBM Plex Mono, and Liberation
        // Mono fonts, but not on Proggy Clean. Pixel peeping in RenderDoc
        // showed a consistent 1px or sometimes 2px error in some characters,
        // e.g. for 'r' and 'i' in Liberation Mono, 'i' is rendered one pixel
        // higher than expected (or 'r' is rendered one pixel lower). Weirdly,
        // they have the same ymin (zero) in metrics returned by fontdue, even
        // though they are visibly offset in the rasterized atlas. Could this be
        // an error in fontdue - either in metrics, or rasterization?

        // TODO(yan): Do we need to render fonts snapped to pixels, or do we
        // just use bilinear blending to smooth them out?

        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let next_draw_primitive_idx = self.ui.draw_primitives.len();

        let parent = &mut self.ui.tree[build_parent_idx];

        assert!(parent.draw_range.end == next_draw_primitive_idx);

        // NB: Vertical align only makes sense, if there is any free space to
        // align in. If we are going to shrink/resize, there is no free space
        // and it simplifies things for us to align to start and not care later.
        //
        // Note that horizontal align still makes sense for shrinking, because
        // the lines will still be jagged and the width difference between
        // longest line and current line will provide the alignment space.
        let vertical_align = if parent.flags.intersects(VERTICAL_RESIZE_FLAGS) {
            Align::Start
        } else {
            vertical_align
        };

        // NB: We zero X and Y of the default parent rect, because emiting draw
        // commands insider a control already uses that control's transform. Not
        // zeroing would apply them twice.
        let available_rect = available_rect
            .unwrap_or_else(|| Rect::new(0.0, 0.0, parent.rect.width, parent.rect.height))
            .inset(inset_amount);
        let available_width = available_rect.width;
        let available_height = available_rect.height;

        // If we are expected to wrap text, but there's not enough space to
        // render a missing character, don't attempt anything.
        if wrap != Wrap::None
            && self.ui.font_atlas.missing_glyph_info().advance_width > available_width
        {
            return;
        }

        struct Line {
            range: Range<usize>,
            width: f32,
        }

        // TODO(yan): @Memory If the allocator is a bump allocator, we
        // potentially prevent it from reclaiming memory if draw_primitives
        // grow.
        let mut lines: Vec<Line, _> = Vec::new_in(&self.ui.allocator);

        let mut last_char_was_whitespace = false;
        let mut begun_word: bool;
        let mut begun_word_start = 0;

        let mut line_range = 0..0;
        let mut line_width = 0.0;

        for (i, c) in text.char_indices() {
            begun_word = !c.is_whitespace();
            if last_char_was_whitespace && !c.is_whitespace() {
                begun_word_start = i;
            }
            last_char_was_whitespace = c.is_whitespace();

            if c == '\n' && !line_range.is_empty() {
                // Note that this could be an empty line, but that's fine.
                lines.push(Line {
                    range: line_range,
                    width: line_width,
                });

                // 1 is the byte width of the '\n', so i + 1 is ok.
                line_range = i + 1..i + 1;
                line_width = 0.0;

                continue;
            }

            let glyph_info = {
                let info = self.ui.font_atlas.glyph_info(c);

                // If we are expected to wrap text, but there's not enough space
                // to render our current character, use metrics for the
                // replacement character instead.
                if wrap != Wrap::None && info.advance_width > available_width {
                    self.ui.font_atlas.missing_glyph_info()
                } else {
                    info
                }
            };
            let glyph_advance_width = glyph_info.advance_width;

            if line_width + glyph_advance_width > available_width {
                match wrap {
                    Wrap::Word => {
                        let begun_word_width = if begun_word {
                            let slice = &text[begun_word_start..i];

                            let mut width = 0.0;
                            for c in slice.chars() {
                                width += self.ui.font_atlas.glyph_info(c).advance_width;
                            }

                            width
                        } else {
                            0.0
                        };

                        if !begun_word || begun_word_width + glyph_advance_width > available_width {
                            // If we are not inside a word right now, or the
                            // begun word is wide enough to cause wrapping by
                            // itself, fall back to letter wrapping.
                            lines.push(Line {
                                range: line_range,
                                width: line_width,
                            });

                            line_range = i..i + c.len_utf8();
                            line_width = glyph_advance_width;
                        } else {
                            // Otherwise commit previous line and move the word
                            // to the next.
                            lines.push(Line {
                                range: line_range.start..begun_word_start,
                                width: line_width - begun_word_width,
                            });

                            line_range = begun_word_start..i + c.len_utf8();
                            line_width = begun_word_width + glyph_advance_width;
                        }

                        continue;
                    }
                    Wrap::Letter => {
                        lines.push(Line {
                            range: line_range,
                            width: line_width,
                        });

                        line_range = i..i + c.len_utf8();
                        line_width = glyph_advance_width;

                        continue;
                    }
                    Wrap::None => (),
                }
            }

            line_range.end += c.len_utf8();
            line_width += glyph_advance_width;
        }

        lines.push(Line {
            range: line_range,
            width: line_width,
        });

        //
        // Trim whitespace.
        //
        // Shorten ranges and decrease widths. The widths can only be decreased
        // here, because the lines were already split and the whitespace widths
        // already contributed to computing text wrap.
        for line in &mut lines {
            let line_slice = &text[line.range.clone()];

            let mut start = line.range.start;
            let mut end = line.range.end;
            let mut trim_width = 0.0;

            for c in line_slice.chars() {
                if !c.is_whitespace() {
                    break;
                }

                start += c.len_utf8();
                trim_width += self.ui.font_atlas.glyph_info(c).advance_width;
            }

            let mut rev_iter = line_slice.chars().rev().peekable();
            while let Some(c) = rev_iter.next() {
                if !c.is_whitespace() {
                    break;
                }

                if rev_iter.peek().is_some() {
                    end -= c.len_utf8();
                    trim_width += self.ui.font_atlas.glyph_info(c).advance_width;
                }
            }

            if start > end {
                start = end;
            }

            line.range.start = start;
            line.range.end = end;
            line.width = f32::max(line.width - trim_width, 0.0)
        }

        //
        // Emit rects based on generated line data.
        //
        let line_metrics = self.ui.font_atlas.font_horizontal_line_metrics();
        let font_scale_factor = self.ui.font_atlas.font_scale_factor();
        let (atlas_width, atlas_height) = {
            let atlas_size = self.ui.font_atlas.image_size();
            (f32::from(atlas_size.0), f32::from(atlas_size.1))
        };
        let (atlas_cell_width, atlas_cell_height) = {
            let atlas_cell_size = self.ui.font_atlas.grid_cell_size();
            (f32::from(atlas_cell_size.0), f32::from(atlas_cell_size.1))
        };

        let mut position_y = if lines.len() as f32 * line_metrics.new_line_size < available_height {
            match vertical_align {
                Align::Start => line_metrics.line_gap + available_rect.y,
                Align::Center => {
                    let line_gap = line_metrics.line_gap;
                    let new_line_size = line_metrics.new_line_size;
                    let text_block_size = new_line_size * lines.len() as f32 - line_gap;

                    line_gap + available_rect.y + (available_height - text_block_size) / 2.0
                }
                Align::End => {
                    let line_gap = line_metrics.line_gap;
                    let new_line_size = line_metrics.new_line_size;
                    let text_block_size = new_line_size * lines.len() as f32 - line_gap;

                    line_gap + available_rect.y + available_height - text_block_size
                }
            }
        } else {
            line_metrics.line_gap
        };

        for line in &lines {
            let line_slice = &text[line.range.clone()];

            let mut position_x = match horizontal_align {
                Align::Start => available_rect.x,
                Align::Center => available_rect.x + (available_width - line.width) / 2.0,
                Align::End => available_rect.x + available_width - line.width,
            };

            for c in line_slice.chars() {
                let info = self.ui.font_atlas.glyph_info(c);

                let rect = Rect::new(
                    position_x + info.xmin,
                    position_y + line_metrics.ascent - info.height - info.ymin,
                    info.width,
                    info.height,
                );

                let texture_rect = Rect::new(
                    f32::from(info.grid_x) * atlas_cell_width / atlas_width,
                    f32::from(info.grid_y) * atlas_cell_height / atlas_height,
                    info.width * font_scale_factor / atlas_width,
                    info.height * font_scale_factor / atlas_height,
                );

                // TODO(yan): @Speed @Memory Does early software scissor make
                // sense here? We also do it later, when translating to the
                // low-level draw list, but we could have less things to
                // translate.
                self.ui.draw_primitives.push(DrawPrimitive::Rect {
                    rect,
                    texture_rect,
                    texture_id: self.ui.font_atlas_texture_id,
                    color,
                });

                parent.draw_range.end += 1;
                if include_in_inline_content_rect {
                    if let Some(inline_content_rect) = &mut parent.inline_content_rect {
                        *inline_content_rect = inline_content_rect.extend_by_rect(rect);
                    } else {
                        parent.inline_content_rect = Some(rect);
                    }
                }

                position_x += info.advance_width;
            }

            position_y += line_metrics.new_line_size;
        }

        // NB: Because this isn't real padding/border, we need to ensure that if
        // we used inset, the final content rect reflects that. This happens
        // automatically for top and left, but we need to add the inset_amount
        // to its size.
        if include_in_inline_content_rect {
            if let Some(inline_content_rect) = &mut parent.inline_content_rect {
                *inline_content_rect = inline_content_rect.resize(Vec2::splat(inset_amount));
            }
        }
    }
}
