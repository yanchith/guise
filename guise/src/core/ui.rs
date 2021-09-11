use core::ops::{BitOr, BitOrAssign, Range};

use alloc::vec::Vec;

use arrayvec::{ArrayString, ArrayVec};

use crate::core::draw_list::DrawList;
use crate::core::font_atlas::{FontAtlas, UnicodeRangeFlags};
use crate::core::math::{Rect, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Inputs(u32);

impl Inputs {
    pub const MOUSE_BUTTON_LEFT: Self = Self(0x01);
    pub const MOUSE_BUTTON_RIGHT: Self = Self(0x02);
    pub const MOUSE_BUTTON_MIDDLE: Self = Self(0x04);
    pub const MOUSE_BUTTON_4: Self = Self(0x08);
    pub const MOUSE_BUTTON_5: Self = Self(0x10);
    pub const MOUSE_BUTTON_6: Self = Self(0x20);
    pub const MOUSE_BUTTON_7: Self = Self(0x40);

    pub const KEYBOARD_TAB: Self = Self(0x80);
    pub const KEYBOARD_LEFT_ARROW: Self = Self(0x100);
    pub const KEYBOARD_RIGHT_ARROW: Self = Self(0x200);
    pub const KEYBOARD_UP_ARROW: Self = Self(0x400);
    pub const KEYBOARD_DOWN_ARROW: Self = Self(0x800);
    pub const KEYBOARD_PAGE_UP: Self = Self(0x1000);
    pub const KEYBOARD_PAGE_DOWN: Self = Self(0x2000);
    pub const KEYBOARD_HOME: Self = Self(0x4000);
    pub const KEYBOARD_END: Self = Self(0x8000);
    pub const KEYBOARD_INSERT: Self = Self(0x10000);
    pub const KEYBOARD_DELETE: Self = Self(0x20000);
    pub const KEYBOARD_BACKSPACE: Self = Self(0x40000);
    pub const KEYBOARD_ENTER: Self = Self(0x80000);
    pub const KEYBOARD_ESCAPE: Self = Self(0x100000);

    // TODO(yan): Fill in gamepad thingies.

    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(
        Self::MOUSE_BUTTON_LEFT.0
            | Self::MOUSE_BUTTON_RIGHT.0
            | Self::MOUSE_BUTTON_MIDDLE.0
            | Self::MOUSE_BUTTON_4.0
            | Self::MOUSE_BUTTON_5.0
            | Self::MOUSE_BUTTON_6.0
            | Self::MOUSE_BUTTON_7.0
            | Self::KEYBOARD_TAB.0
            | Self::KEYBOARD_LEFT_ARROW.0
            | Self::KEYBOARD_RIGHT_ARROW.0
            | Self::KEYBOARD_UP_ARROW.0
            | Self::KEYBOARD_DOWN_ARROW.0
            | Self::KEYBOARD_PAGE_UP.0
            | Self::KEYBOARD_PAGE_DOWN.0
            | Self::KEYBOARD_HOME.0
            | Self::KEYBOARD_END.0
            | Self::KEYBOARD_INSERT.0
            | Self::KEYBOARD_DELETE.0
            | Self::KEYBOARD_BACKSPACE.0
            | Self::KEYBOARD_ENTER.0
            | Self::KEYBOARD_ESCAPE.0,
    );

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

impl BitOr for Inputs {
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

// TODO(yan): @Speed Layout::None that does nothing or asserts for child
// controls?
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
enum DrawCommand {
    DrawRect {
        position_rect: Rect,
        tex_coord_rect: Rect,
        color: u32,
        texture_id: u64,
    },
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

    pub const NONE: Self = Self(0);
    pub const ALL: Self =
        Self(Self::CAPTURE_SCROLL.0 | Self::CAPTURE_HOVER.0 | Self::CAPTURE_ACTIVE.0);

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

impl BitOr for CtrlFlags {
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

    // TODO(yan): @Speed @Memory Make indices more compact. Option<usize> (16 bytes!!!)
    // -> Idx(U32) (4 bytes), where Idx(u32::MAX) is the sentinel value for
    // None.
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

    layout_cache_resolved_position: Vec2,
    layout_cache_content_extents: Vec2,
}

pub struct Ui {
    draw_commands: Vec<DrawCommand>,
    draw_list: DrawList,

    font_atlas: FontAtlas,
    font_atlas_texture_id: u64,

    tree: Vec<CtrlNode>,
    tree_root_idx: usize,

    build_parent_idx: Option<usize>,
    build_sibling_idx: Option<usize>,

    render_target_extents: Vec2,

    current_frame: u32,

    window_scroll_delta: Vec2,
    window_cursor_position: Vec2,
    window_inputs_pressed: Inputs,
    window_inputs_released: Inputs,
    window_received_characters: ArrayString<32>,

    active_ctrl_idx: Option<usize>,
    hovered_ctrl_idx: Option<usize>,
    hovered_capturing_ctrl_idx: Option<usize>,
}

impl Ui {
    pub fn new(
        render_target_width: f32,
        render_target_height: f32,
        font_bytes: &[u8],
        font_unicode_range_flags: UnicodeRangeFlags,
        font_size: f32,
    ) -> Self {
        let render_target_extents = Vec2::new(render_target_width, render_target_height);
        let font_atlas = FontAtlas::new(font_bytes, font_unicode_range_flags, font_size);

        let mut tree = Vec::with_capacity(128);
        let tree_root_idx = 0;
        tree.push(CtrlNode {
            id: 0,

            parent_idx: None,
            child_idx: None,
            sibling_idx: None,

            last_frame: 0,
            last_frame_in_active_path: 0,

            flags: CtrlFlags::NONE,
            layout: Layout::Free,
            rect: Rect::from_points(Vec2::ZERO, render_target_extents),
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

            layout_cache_resolved_position: Vec2::ZERO,
            layout_cache_content_extents: Vec2::ZERO,
        });

        Self {
            draw_commands: Vec::new(),
            draw_list: DrawList::new(),

            font_atlas,
            font_atlas_texture_id: 0,

            tree,
            tree_root_idx,

            build_parent_idx: None,
            build_sibling_idx: None,

            render_target_extents,

            current_frame: 0,

            window_scroll_delta: Vec2::ZERO,
            window_cursor_position: Vec2::ZERO,
            window_inputs_pressed: Inputs::empty(),
            window_inputs_released: Inputs::empty(),
            window_received_characters: ArrayString::new(),

            active_ctrl_idx: None,
            hovered_ctrl_idx: None,
            hovered_capturing_ctrl_idx: None,
        }
    }

    pub fn set_render_target_extents(
        &mut self,
        render_target_width: f32,
        render_target_height: f32,
    ) {
        self.render_target_extents = Vec2::new(render_target_width, render_target_height);
    }

    pub fn set_font_atlas_texture_id(&mut self, font_atlas_texture_id: u64) {
        self.font_atlas_texture_id = font_atlas_texture_id;
    }

    pub fn add_window_scroll_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.window_scroll_delta += Vec2::new(delta_x, delta_y);
    }

    pub fn set_window_cursor_position(&mut self, cursor_x: f32, cursor_y: f32) {
        self.window_cursor_position = Vec2::new(cursor_x, cursor_y);
    }

    pub fn add_window_inputs_pressed(&mut self, inputs: Inputs) {
        self.window_inputs_pressed |= inputs;
    }

    pub fn add_window_inputs_released(&mut self, inputs: Inputs) {
        self.window_inputs_released |= inputs;
    }

    pub fn add_window_character(&mut self, character: char) {
        #[allow(clippy::single_match)]
        match self.window_received_characters.try_push(character) {
            Ok(()) => (/* We pushed the character */),
            Err(_) => (/* We loose chars here, but such is life */),
        }
    }

    pub fn font_atlas_image_extents(&self) -> (u16, u16) {
        self.font_atlas.image_extents()
    }

    pub fn font_atlas_image_rgba8_unorm(&self) -> &[u8] {
        self.font_atlas.image_rgba8_unorm()
    }

    pub fn ctrl_count(&self) -> usize {
        self.tree.len()
    }

    pub fn draw_list(&self) -> &DrawList {
        &self.draw_list
    }

    pub fn begin_frame(&mut self) -> Frame<'_> {
        self.draw_commands.clear();
        self.draw_list.clear();

        self.current_frame = self.current_frame.wrapping_add(1);

        let tree_root_ctrl = &mut self.tree[self.tree_root_idx];
        tree_root_ctrl.last_frame = self.current_frame;
        tree_root_ctrl.last_frame_in_active_path = self.current_frame;
        tree_root_ctrl.rect = Rect::from_points(Vec2::ZERO, self.render_target_extents);

        //
        // Scroll a control.
        //
        // If the hovered control doesn't want scrolling or doesn't have
        // overflow it could scroll, walk the tree up to the first eligible
        // control and scroll that!
        if self.window_scroll_delta != Vec2::ZERO {
            if let Some(idx) = self.hovered_ctrl_idx {
                let mut ctrl = &mut self.tree[idx];
                let mut ctrl_scroll_extents = Vec2::ZERO.max(
                    ctrl.layout_cache_content_extents - ctrl.rect.extents()
                        + 2.0 * Vec2::new(ctrl.padding, ctrl.padding)
                        + 2.0 * Vec2::new(ctrl.border, ctrl.border),
                );
                let mut ctrl_scroll_offset_new = (ctrl.scroll_offset - self.window_scroll_delta)
                    .clamp(Vec2::ZERO, ctrl_scroll_extents);
                let mut ctrl_can_scroll = ctrl.flags.intersects(CtrlFlags::CAPTURE_SCROLL)
                    && ctrl_scroll_offset_new != ctrl.scroll_offset;

                while !ctrl_can_scroll && ctrl.parent_idx.is_some() {
                    let parent_idx = ctrl.parent_idx.unwrap();

                    ctrl = &mut self.tree[parent_idx];
                    ctrl_scroll_extents = Vec2::ZERO.max(
                        ctrl.layout_cache_content_extents - ctrl.rect.extents()
                            + 2.0 * Vec2::new(ctrl.padding, ctrl.padding)
                            + 2.0 * Vec2::new(ctrl.border, ctrl.border),
                    );
                    ctrl_scroll_offset_new = (ctrl.scroll_offset - self.window_scroll_delta)
                        .clamp(Vec2::ZERO, ctrl_scroll_extents);
                    ctrl_can_scroll = ctrl.flags.intersects(CtrlFlags::CAPTURE_SCROLL)
                        && ctrl_scroll_offset_new != ctrl.scroll_offset;
                }

                if ctrl_can_scroll {
                    ctrl.scroll_offset = ctrl_scroll_offset_new;
                }
            }
        }

        //
        // Find hovered control.
        //
        // Look at the tree starting from the root and follow branches where the
        // child control's rect contains the cursor.
        self.hovered_ctrl_idx = find_hovered_ctrl(
            &self.tree,
            self.tree_root_idx,
            Rect::from_points(Vec2::ZERO, self.render_target_extents),
            self.window_cursor_position,
        );
        self.hovered_capturing_ctrl_idx = None;

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
            }
        }

        fn find_hovered_ctrl(
            tree: &[CtrlNode],
            ctrl_idx: usize,
            ctrl_rect: Rect,
            cursor_position: Vec2,
        ) -> Option<usize> {
            let ctrl = &tree[ctrl_idx];
            if ctrl_rect.contains_point(cursor_position) {
                if ctrl.layout == Layout::Free {
                    // For free layout, we'd like to preserve the render order
                    // of controls when determining hover. The most recently
                    // active control (on top) has priority when determining
                    // hover, followed by the next most recently active control,
                    // all the way up to the least recently active control.
                    //
                    // TODO(yan): @Correctness Would be great if we didn't panic
                    // in ArrayVec::push.

                    let child_rect_offset_base = ctrl_rect.min_point() - ctrl.scroll_offset;

                    let mut siblings: ArrayVec<(usize, u32), 64> = ArrayVec::new();
                    if let Some(child_idx) = ctrl.child_idx {
                        let mut ctrl = &tree[child_idx];

                        siblings.push((child_idx, ctrl.last_frame_in_active_path));

                        while let Some(sibling_idx) = ctrl.sibling_idx {
                            ctrl = &tree[sibling_idx];
                            siblings.push((sibling_idx, ctrl.last_frame_in_active_path));
                        }
                    }

                    siblings.sort_unstable_by_key(|&(_, frame)| frame);

                    for (sibling_idx, _) in siblings.into_iter().rev() {
                        let child_idx = sibling_idx;
                        let child = &tree[sibling_idx];

                        let hovered_ctrl = find_hovered_ctrl(
                            tree,
                            child_idx,
                            child.rect + child_rect_offset_base,
                            cursor_position,
                        );

                        if hovered_ctrl.is_some() {
                            // This control is hovered, but also one of its
                            // children is.
                            return hovered_ctrl;
                        }
                    }

                    // This control is hovered, but none of its children are.
                    Some(ctrl_idx)
                } else if let Some(child_idx) = ctrl.child_idx {
                    let child_rect_offset_base = ctrl_rect.min_point()
                        + Vec2::new(ctrl.padding, ctrl.padding)
                        + Vec2::new(ctrl.border, ctrl.border)
                        - ctrl.scroll_offset;

                    let mut child = &tree[child_idx];

                    let mut hovered_ctrl = find_hovered_ctrl(
                        tree,
                        child_idx,
                        child.rect + child_rect_offset_base,
                        cursor_position,
                    );

                    if hovered_ctrl.is_some() {
                        // This control is hovered, but also one of its
                        // children is.
                        return hovered_ctrl;
                    }

                    let mut position = match ctrl.layout {
                        Layout::Free => unreachable!(),
                        Layout::Horizontal => {
                            child.rect.x() + child.rect.offset(child.margin).width()
                        }
                        Layout::Vertical => {
                            child.rect.y() + child.rect.offset(child.margin).height()
                        }
                    };

                    while let Some(sibling_idx) = child.sibling_idx {
                        child = &tree[sibling_idx];
                        let child_rect_offset = match ctrl.layout {
                            Layout::Free => unreachable!(),
                            Layout::Horizontal => Vec2::X * position,
                            Layout::Vertical => Vec2::Y * position,
                        };

                        hovered_ctrl = find_hovered_ctrl(
                            tree,
                            sibling_idx,
                            child.rect + child_rect_offset_base + child_rect_offset,
                            cursor_position,
                        );

                        if hovered_ctrl.is_some() {
                            // This control is hovered, but also one of its
                            // children is.
                            return hovered_ctrl;
                        }

                        match ctrl.layout {
                            Layout::Free => unreachable!(),
                            Layout::Horizontal => {
                                position += child.rect.x();
                                position += child.rect.offset(child.margin).width();
                            }
                            Layout::Vertical => {
                                position += child.rect.y();
                                position += child.rect.offset(child.margin).height();
                            }
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

        self.build_parent_idx = Some(self.tree_root_idx);
        self.build_sibling_idx = None;

        Frame { ui: self }
    }

    pub fn end_frame(&mut self) {
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

        // TODO(yan): @Speed This garbage collection is terrible! Can we do better?
        const RELOCATIONS_CAP: usize = 256;
        let mut relocations: ArrayVec<(usize, usize), RELOCATIONS_CAP> = ArrayVec::new();

        fn apply_relocation(idx_to_relocate: &mut Option<usize>, from: usize, to: usize) {
            if let Some(idx) = idx_to_relocate.as_mut() {
                if *idx == from {
                    *idx = to;
                }
            }
        }

        let mut ctrl_idx = 0;
        while ctrl_idx < self.tree.len() {
            if self.tree[ctrl_idx].last_frame == self.current_frame {
                ctrl_idx += 1;
            } else {
                // The swapped in control could be dead too. Keep doing
                // swap_remove until we find a live control, only then record
                // the relocation.
                while ctrl_idx < self.tree.len()
                    && self.tree[ctrl_idx].last_frame != self.current_frame
                {
                    self.tree.swap_remove(ctrl_idx);
                    ctrl_idx += 1;
                }

                relocations.push((ctrl_idx, self.tree.len()));

                // Our relocations buffer might have filled. Relocation time!
                if relocations.len() == RELOCATIONS_CAP {
                    for ctrl in &mut self.tree {
                        // Only apply relocations to live controls.
                        if ctrl.last_frame == self.current_frame {
                            for relocation in &relocations {
                                let &(from, to) = relocation;
                                apply_relocation(&mut ctrl.parent_idx, from, to);
                                apply_relocation(&mut ctrl.child_idx, from, to);
                                apply_relocation(&mut ctrl.sibling_idx, from, to);
                                apply_relocation(&mut self.active_ctrl_idx, from, to);
                            }
                        }
                    }

                    relocations.clear();
                }
            }
        }

        // Apply the rest of the relocations, if any.
        if !relocations.is_empty() {
            for ctrl in &mut self.tree {
                for relocation in &relocations {
                    let &(from, to) = relocation;
                    apply_relocation(&mut ctrl.parent_idx, from, to);
                    apply_relocation(&mut ctrl.child_idx, from, to);
                    apply_relocation(&mut ctrl.sibling_idx, from, to);
                    apply_relocation(&mut self.active_ctrl_idx, from, to);
                }
            }
        }

        //
        // Update layout.
        //
        // Because the build phase is now done, we can incorporate all the
        // layout changes. They will be used for this frame's render phase, and
        // next frame's build phase.
        update_ctrl_layout(&mut self.tree, self.tree_root_idx, Vec2::ZERO);

        fn update_ctrl_layout(
            tree: &mut [CtrlNode],
            ctrl_idx: usize,
            ctrl_resolved_position_base: Vec2,
        ) {
            let ctrl = &tree[ctrl_idx];
            let ctrl_layout = ctrl.layout;
            let ctrl_inline_content_rect = ctrl.inline_content_rect;
            let ctrl_resolved_position = ctrl_resolved_position_base + ctrl.rect.min_point();

            if let Some(child_idx) = ctrl.child_idx {
                let child_resolved_position_base = ctrl_resolved_position;

                update_ctrl_layout(tree, child_idx, child_resolved_position_base);

                let mut child = &tree[child_idx];
                let mut child_margin_rect = child.rect.offset(child.margin);

                let mut child_resolved_position_offset = match ctrl_layout {
                    Layout::Free => Vec2::ZERO,
                    Layout::Horizontal => Vec2::X * child_margin_rect.width(),
                    Layout::Vertical => Vec2::Y * child_margin_rect.height(),
                };

                let mut min_point = child_margin_rect.min_point();
                let mut max_point = child_margin_rect.max_point();

                while let Some(sibling_idx) = child.sibling_idx {
                    update_ctrl_layout(
                        tree,
                        sibling_idx,
                        child_resolved_position_base + child_resolved_position_offset,
                    );

                    child = &tree[sibling_idx];
                    child_margin_rect = child.rect.offset(child.margin);

                    match ctrl_layout {
                        Layout::Free => {
                            min_point = min_point.min(child_margin_rect.min_point());
                            max_point = max_point.max(child_margin_rect.max_point());
                        }
                        Layout::Horizontal => {
                            child_resolved_position_offset += Vec2::X * child_margin_rect.width();
                            max_point.x += child_margin_rect.width();
                            max_point.y = max_point.y.max(child_margin_rect.max_y());
                        }
                        Layout::Vertical => {
                            child_resolved_position_offset += Vec2::Y * child_margin_rect.height();
                            max_point.x = max_point.x.max(child_margin_rect.max_x());
                            max_point.y += child_margin_rect.height();
                        }
                    }
                }

                if let Some(inline_content_rect) = ctrl_inline_content_rect {
                    min_point = min_point.min(inline_content_rect.min_point());
                    max_point = max_point.max(inline_content_rect.max_point());
                }

                let ctrl_mut = &mut tree[ctrl_idx];
                ctrl_mut.layout_cache_resolved_position = ctrl_resolved_position;
                ctrl_mut.layout_cache_content_extents = max_point - min_point;
            } else {
                let ctrl_mut = &mut tree[ctrl_idx];

                ctrl_mut.layout_cache_resolved_position = ctrl_resolved_position;
                if let Some(inline_content_rect) = ctrl_inline_content_rect {
                    tree[ctrl_idx].layout_cache_content_extents = inline_content_rect.extents();
                } else {
                    tree[ctrl_idx].layout_cache_content_extents = Vec2::ZERO;
                }
            }
        }

        //
        // Render into the draw lists.
        //
        render(
            &self.tree,
            self.tree_root_idx,
            Rect::from_points(Vec2::ZERO, self.render_target_extents),
            Rect::from_points(Vec2::ZERO, self.render_target_extents),
            &self.draw_commands,
            self.font_atlas_texture_id,
            &mut self.draw_list,
        );

        fn render(
            tree: &[CtrlNode],
            ctrl_idx: usize,
            ctrl_rect: Rect,
            ctrl_scissor_rect: Rect,
            draw_commands: &[DrawCommand],
            font_atlas_texture_id: u64,
            draw_list: &mut DrawList,
        ) {
            let ctrl = &tree[ctrl_idx];

            if ctrl.draw_self {
                let border_color = ctrl.draw_self_border_color;
                let background_color = ctrl.draw_self_background_color;

                let ctrl_padding_rect = ctrl_rect.inset(ctrl.border);

                if !ctrl_rect.is_empty() && !ctrl_padding_rect.is_empty() {
                    // Note that `.max(0.0)` is used in
                    // substractions here because fp precision
                    // commonly caused the result to be below 0,
                    // which is a big no-no for Rect::new.

                    let outer = ctrl_rect;
                    let inner = ctrl_padding_rect;

                    let lx = outer.x();
                    let ly = outer.y();
                    let lwidth = (inner.x() - outer.x()).max(0.0);
                    let lheight = outer.height();
                    let left = Rect::new(lx, ly, lwidth, lheight);

                    let tx = inner.x();
                    let ty = outer.y();
                    let twidth = inner.width();
                    let theight = (inner.y() - outer.y()).max(0.0);
                    let top = Rect::new(tx, ty, twidth, theight);

                    let rx = inner.x() + inner.width();
                    let ry = outer.y();
                    let rwidth = (outer.width() - inner.width() - lwidth).max(0.0);
                    let rheight = outer.height();
                    let right = Rect::new(rx, ry, rwidth, rheight);

                    let bx = inner.x();
                    let by = inner.y() + inner.height();
                    let bwidth = inner.width();
                    let bheight = (outer.height() - inner.height() - theight).max(0.0);
                    let bottom = Rect::new(bx, by, bwidth, bheight);

                    if !left.is_empty() {
                        draw_list.draw_rect(left, Rect::ZERO, border_color, font_atlas_texture_id);
                    }

                    if !top.is_empty() {
                        draw_list.draw_rect(top, Rect::ZERO, border_color, font_atlas_texture_id);
                    }

                    if !right.is_empty() {
                        draw_list.draw_rect(right, Rect::ZERO, border_color, font_atlas_texture_id);
                    }

                    if !bottom.is_empty() {
                        draw_list.draw_rect(
                            bottom,
                            Rect::ZERO,
                            border_color,
                            font_atlas_texture_id,
                        );
                    }
                }

                draw_list.draw_rect(
                    ctrl_padding_rect,
                    Rect::ZERO,
                    background_color,
                    font_atlas_texture_id,
                );
            }

            draw_list.push_scissor_rect(ctrl_scissor_rect);

            for draw_command_idx in ctrl.draw_range.clone() {
                let draw_command = &draw_commands[draw_command_idx];

                match draw_command {
                    DrawCommand::DrawRect {
                        position_rect,
                        tex_coord_rect,
                        color,
                        texture_id,
                    } => {
                        let rect = *position_rect
                            + ctrl_rect.min_point()
                            + Vec2::new(ctrl.border, ctrl.border)
                            + Vec2::new(ctrl.padding, ctrl.padding)
                            - ctrl.scroll_offset;

                        draw_list.draw_rect(rect, *tex_coord_rect, *color, *texture_id);
                    }
                }
            }

            if ctrl.layout == Layout::Free {
                // For free layout, we'd like to preserve render order of
                // controls, e.g. we render least recently active control first,
                // then a more recently active control, all the way up to the
                // currently active control. The thing is sorting the entire
                // sibling list can become quite expensive (and stack size is
                // limited), so we cap the sorting to N controls. If we exceed
                // that, we gracefully degrade from fully sorted rendering to
                // just rendering the active control last, and the rest in
                // definition order.
                //
                // TODO(yan): And by graceful degradation, I mean something else
                // than the currently panicking code below (ArrayVec::push).

                let child_rect_offset_base = ctrl_rect.min_point() - ctrl.scroll_offset;

                let mut siblings: ArrayVec<(usize, u32), 64> = ArrayVec::new();
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
                    let child = &tree[sibling_idx];

                    // Clamp the resolved rect to ctrl_rect and do not render
                    // the child control unless the clamped rect has nonzero
                    // area. This is because some renderers dislike scissor
                    // rects that have zero dimensions or ones that escape the
                    // screen. We ensure this by having the topmost element of
                    // the same size as the render target.
                    // rect_clamped_has_dims checks, whether the dimensions
                    // won't round to zero.
                    let rect = child.rect + child_rect_offset_base;
                    let scissor_rect = ctrl_scissor_rect.clamp_rect(rect).inset(child.border);
                    let scissor_rect_has_dims =
                        scissor_rect.width() >= 0.5 && scissor_rect.height() >= 0.5;

                    if ctrl_rect.intersects_rect(rect) && scissor_rect_has_dims {
                        render(
                            tree,
                            sibling_idx,
                            rect,
                            scissor_rect,
                            draw_commands,
                            font_atlas_texture_id,
                            draw_list,
                        );
                    }
                }
            } else {
                // For horizontal and vertical layouts, we don't need any
                // sorting and just iterate over the controls in definition
                // order.
                if let Some(child_idx) = ctrl.child_idx {
                    let child_rect_offset_base = ctrl_rect.min_point()
                        + Vec2::new(ctrl.border, ctrl.border)
                        + Vec2::new(ctrl.padding, ctrl.padding)
                        - ctrl.scroll_offset;

                    let mut child = &tree[child_idx];

                    {
                        // Same as above, we clamp the control's rect so that we
                        // don't output an invalid scissor rect.
                        let rect = child.rect + child_rect_offset_base;
                        let scissor_rect = ctrl_scissor_rect.clamp_rect(rect).inset(child.border);
                        let scissor_rect_has_dims =
                            scissor_rect.width() >= 0.5 && scissor_rect.height() >= 0.5;

                        if ctrl_rect.intersects_rect(rect) && scissor_rect_has_dims {
                            render(
                                tree,
                                child_idx,
                                rect,
                                scissor_rect,
                                draw_commands,
                                font_atlas_texture_id,
                                draw_list,
                            );
                        }
                    }

                    let mut position = match ctrl.layout {
                        Layout::Free => unreachable!(),
                        Layout::Horizontal => {
                            child.rect.x() + child.rect.offset(child.margin).width()
                        }
                        Layout::Vertical => {
                            child.rect.y() + child.rect.offset(child.margin).height()
                        }
                    };

                    while let Some(sibling_idx) = child.sibling_idx {
                        child = &tree[sibling_idx];
                        let child_rect_offset = match ctrl.layout {
                            Layout::Free => unreachable!(),
                            Layout::Horizontal => Vec2::X * position,
                            Layout::Vertical => Vec2::Y * position,
                        };

                        // Same as above, we clamp the control's rect so that we
                        // don't output an invalid scissor rect.
                        let rect = child.rect + child_rect_offset_base + child_rect_offset;
                        let scissor_rect = ctrl_scissor_rect.clamp_rect(rect).inset(child.border);
                        let scissor_rect_has_dims =
                            scissor_rect.width() >= 0.5 && scissor_rect.height() >= 0.5;

                        if ctrl_rect.intersects_rect(rect) && scissor_rect_has_dims {
                            render(
                                tree,
                                sibling_idx,
                                rect,
                                scissor_rect,
                                draw_commands,
                                font_atlas_texture_id,
                                draw_list,
                            );
                        }

                        match ctrl.layout {
                            Layout::Free => unreachable!(),
                            Layout::Horizontal => {
                                position += child.rect.x();
                                position += child.rect.offset(child.margin).width();
                            }
                            Layout::Vertical => {
                                position += child.rect.y();
                                position += child.rect.offset(child.margin).height();
                            }
                        }
                    }
                }
            }

            draw_list.pop_scissor_rect();
        }

        self.build_parent_idx = None;
        self.build_sibling_idx = None;

        self.window_scroll_delta = Vec2::ZERO;
        self.window_inputs_pressed = Inputs::empty();
        self.window_inputs_released = Inputs::empty();
        self.window_received_characters.clear();
    }
}

pub struct Frame<'a> {
    ui: &'a mut Ui,
}

impl<'a> Frame<'a> {
    pub fn push_ctrl(&mut self, id: u32) -> Ctrl<'_> {
        // Push a control onto the tree. The control can either be completely new,
        // or already present in the tree from previous frame. Controls are
        // identified by their ID, which has to be unique among siblings of a
        // single control.
        //
        // Whether inserting a new control or updating an existing one, we must
        // never unlink a control already present in the tree, because it may
        // yet be updated later this frame and we would have lost its state.
        //
        // Updated controls are first unlinked from the tree, and then
        // re-inserted at the current position. This means that dead controls
        // (if any) will be located after the live controls once the UI is
        // built.
        //
        // Current position in the tree is tracked by two indices:
        // build_parent_idx and build_sibling_idx, pointing to the current
        // parent and last inserted sibling, respectively.
        //
        // Note: Changing the control's layout options invalidates the layout
        // from last frame for this control and all its sibling successors, and
        // their children, but so does re-ordering, or not updating control from
        // earlier frame. The layout will become valid on the next frame once
        // again.

        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let draw_range = {
            let next_draw_command_idx = self.ui.draw_commands.len();
            next_draw_command_idx..next_draw_command_idx
        };

        let result = {
            let parent = &self.ui.tree[build_parent_idx];

            // TODO(yan): @Speed This is quadratic behavior. Not great.
            if let Some(child_idx) = parent.child_idx {
                let mut prev_ctrl_idx = None;
                let mut ctrl_idx = child_idx;
                let mut ctrl = &mut self.ui.tree[child_idx];

                if ctrl.id == id {
                    Some((ctrl_idx, prev_ctrl_idx))
                } else {
                    let mut current_idx_res = None;

                    while let Some(sibling_idx) = ctrl.sibling_idx {
                        prev_ctrl_idx = Some(ctrl_idx);
                        ctrl_idx = sibling_idx;
                        ctrl = &mut self.ui.tree[sibling_idx];

                        if ctrl.id == id {
                            current_idx_res = Some((ctrl_idx, prev_ctrl_idx));
                            break;
                        }
                    }

                    current_idx_res
                }
            } else {
                None
            }
        };

        let current_idx = if let Some((found_idx, found_prev_sibling_idx)) = result {
            let ctrl = &mut self.ui.tree[found_idx];

            ctrl.last_frame = self.ui.current_frame;
            ctrl.inline_content_rect = None;
            ctrl.draw_range = draw_range;

            // After updating the control's data, we unlink the nore from its
            // original place and re-link as either the next sibling of the
            // build sibling (if build sibling already exists) or first child of
            // the build parent (if the build sibling doesn't exist yet).
            //
            // IMPORTANT(yan): We do not unlink/re-link, if the control is the
            // build sibling itself as that would create a cycle. Fortunately,
            // the control is already linked correctly in this case.
            if self.ui.build_sibling_idx != Some(found_idx) {
                // Unlink the control from previous sibling or parent (in case
                // the control was the first child).
                if let Some(found_prev_sibling_idx) = found_prev_sibling_idx {
                    self.ui.tree[found_prev_sibling_idx].sibling_idx = ctrl.sibling_idx;
                } else {
                    self.ui.tree[build_parent_idx].child_idx = ctrl.sibling_idx;
                }

                // Re-link the control as next sibling of the build sibling or
                // as first child of build parent (in case there is no build
                // sibling yet).
                if let Some(build_sibling_idx) = self.ui.build_sibling_idx {
                    let build_sibling = &mut self.ui.tree[build_sibling_idx];
                    let build_sibling_next_sibling_idx = build_sibling.sibling_idx;

                    build_sibling.sibling_idx = Some(found_idx);

                    self.ui.tree[found_idx].sibling_idx = build_sibling_next_sibling_idx;
                } else {
                    let build_parent = &mut self.ui.tree[build_parent_idx];
                    let build_parent_child_idx = build_parent.child_idx;

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

                layout_cache_resolved_position: Vec2::ZERO,
                layout_cache_content_extents: Vec2::ZERO,
            });

            idx
        };

        self.ui.build_parent_idx = Some(current_idx);
        self.ui.build_sibling_idx = None;

        Ctrl {
            idx: current_idx,
            ui: &mut self.ui,
        }
    }

    pub fn pop_ctrl(&mut self) {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();

        // Finalize the parent and last inserted sibling controls and clean
        // their indices so that they only reference live controls. If no child
        // controls were inserted, clear the parent's child references (which
        // could contain dead controls from previous frame). Also cut off the
        // dead sibling controls of the last sibling here, so that we won't have
        // to relocate references to them during garbage collection.

        // TODO(yan): @Correctness Assert that push_ctrl and pop_ctrl are
        // parethesized correctly! Count current tree depth and assert something
        // in both pop_ctrl and end_frame?

        let parent = &mut self.ui.tree[build_parent_idx];
        let parent_parent_idx = parent.parent_idx;

        if let Some(build_sibling_idx) = self.ui.build_sibling_idx {
            self.ui.tree[build_sibling_idx].sibling_idx = None;
        } else {
            parent.child_idx = None;
        }

        self.ui.build_parent_idx = parent_parent_idx;
        self.ui.build_sibling_idx = Some(build_parent_idx);
    }

    pub fn window_cursor_position(&self) -> Vec2 {
        self.ui.window_cursor_position
    }

    pub fn window_inputs_pressed(&self) -> Inputs {
        self.ui.window_inputs_pressed
    }

    pub fn window_inputs_released(&self) -> Inputs {
        self.ui.window_inputs_released
    }

    pub fn window_received_characters(&self) -> &str {
        &self.ui.window_received_characters
    }

    pub fn ctrl_inner_extents(&self) -> Vec2 {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let parent = &self.ui.tree[build_parent_idx];
        let rect = parent.rect.inset(parent.border).inset(parent.padding);

        rect.extents()
    }
}

pub struct Ctrl<'a> {
    idx: usize,
    ui: &'a mut Ui,
}

// TODO(yan): Vertical and horizontal align.
impl Ctrl<'_> {
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

    pub fn window_position(&self) -> Vec2 {
        self.ui.tree[self.idx].layout_cache_resolved_position
    }

    pub fn inner_extents(&self) -> Vec2 {
        let ctrl = &self.ui.tree[self.idx];
        let rect = ctrl.rect.inset(ctrl.border).inset(ctrl.padding);

        rect.extents()
    }

    pub fn draw_rect(
        &mut self,
        extend_content_rect: bool,
        position_rect: Rect,
        tex_coord_rect: Rect,
        color: u32,
        texture_id: u64,
    ) {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let next_draw_command_idx = self.ui.draw_commands.len();

        let parent = &mut self.ui.tree[build_parent_idx];
        assert!(parent.draw_range.end == next_draw_command_idx);

        self.ui.draw_commands.push(DrawCommand::DrawRect {
            position_rect,
            tex_coord_rect,
            color,
            texture_id,
        });

        parent.draw_range.end += 1;
        if extend_content_rect {
            parent.inline_content_rect = Some(
                parent
                    .inline_content_rect
                    .map(|r| r.extend_by_rect(position_rect))
                    .unwrap_or(position_rect),
            );
        }
    }

    pub fn draw_text(
        &mut self,
        extend_content_rect: bool,
        position: Vec2,
        text: &str,
        horizontal_align: Align,
        vertical_align: Align,
        wrap: Wrap,
        color: u32,
    ) {
        let build_parent_idx = self.ui.build_parent_idx.unwrap();
        let next_draw_command_idx = self.ui.draw_commands.len();

        let parent = &mut self.ui.tree[build_parent_idx];
        assert!(parent.draw_range.end == next_draw_command_idx);

        let available_rect = parent.rect.inset(parent.border).inset(parent.padding);
        let available_width = (available_rect.width() - position.x).max(0.0);
        let available_height = (available_rect.height() - position.y).max(0.0);

        // If we are expected to wrap text, but there's not enough space to
        // render a missing character, don't attempt anything.
        if wrap != Wrap::None
            && self
                .ui
                .font_atlas
                .missing_glyph_info()
                .metrics
                .advance_width
                > available_width
        {
            return;
        }

        struct Line {
            range: Range<usize>,
            width: f32,
        }

        let mut lines: ArrayVec<Line, 1024> = ArrayVec::new();

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
                if wrap != Wrap::None && info.metrics.advance_width > available_width {
                    self.ui.font_atlas.missing_glyph_info()
                } else {
                    info
                }
            };
            let glyph_advance_width = glyph_info.metrics.advance_width;

            if line_width + glyph_advance_width > available_width {
                match wrap {
                    Wrap::Word => {
                        let begun_word_width = if begun_word {
                            let slice = &text[begun_word_start..i];

                            let mut width = 0.0;
                            for c in slice.chars() {
                                width += self.ui.font_atlas.glyph_info(c).metrics.advance_width;
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
                trim_width += self.ui.font_atlas.glyph_info(c).metrics.advance_width;
            }

            let mut rev_iter = line_slice.chars().rev().peekable();
            while let Some(c) = rev_iter.next() {
                if !c.is_whitespace() {
                    break;
                }

                if rev_iter.peek().is_some() {
                    end -= c.len_utf8();
                    trim_width += self.ui.font_atlas.glyph_info(c).metrics.advance_width;
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
        let line_metrics = self.ui.font_atlas.line_metrics();
        let (atlas_width, atlas_height) = {
            let atlas_extents = self.ui.font_atlas.image_extents();
            (f32::from(atlas_extents.0), f32::from(atlas_extents.1))
        };
        let (atlas_cell_width, atlas_cell_height) = {
            let atlas_cell_extents = self.ui.font_atlas.grid_cell_extents();
            (
                f32::from(atlas_cell_extents.0),
                f32::from(atlas_cell_extents.1),
            )
        };

        let mut position_y = if lines.len() as f32 * line_metrics.new_line_size < available_height {
            match vertical_align {
                Align::Start => position.y,
                Align::Center => {
                    let line_gap = line_metrics.line_gap;
                    let new_line_size = line_metrics.new_line_size;
                    let text_block_size = new_line_size * lines.len() as f32 - line_gap;

                    position.y + (available_height - text_block_size) / 2.0
                }
                Align::End => {
                    let line_gap = line_metrics.line_gap;
                    let new_line_size = line_metrics.new_line_size;
                    let text_block_size = new_line_size * lines.len() as f32 - line_gap;

                    position.y + available_height - text_block_size
                }
            }
        } else {
            0.0
        };

        for line in &lines {
            let line_slice = &text[line.range.clone()];

            let mut position_x = match horizontal_align {
                Align::Start => position.x,
                Align::Center => position.x + (available_width - line.width) / 2.0,
                Align::End => position.x + available_width - line.width,
            };

            for c in line_slice.chars() {
                let glyph_info = self.ui.font_atlas.glyph_info(c);
                let glyph_advance_width = glyph_info.metrics.advance_width as f32;
                let glyph_width = glyph_info.metrics.width as f32;
                let glyph_height = glyph_info.metrics.height as f32;
                let glyph_xmin = glyph_info.metrics.xmin as f32;
                let glyph_ymin = glyph_info.metrics.ymin as f32;

                let position_rect = Rect::new(
                    position_x + glyph_xmin,
                    position_y + line_metrics.ascent - glyph_height - glyph_ymin,
                    glyph_width,
                    glyph_height,
                );

                let tex_coord_rect = Rect::new(
                    f32::from(glyph_info.grid_x) * atlas_cell_width / atlas_width,
                    f32::from(glyph_info.grid_y) * atlas_cell_height / atlas_height,
                    glyph_width / atlas_width,
                    glyph_height / atlas_height,
                );

                // TODO(yan): @Speed @Memory Does early software scissor make
                // sense here? We also do it later, when translating to the
                // low-level draw list, but we could have less things to
                // translate.
                self.ui.draw_commands.push(DrawCommand::DrawRect {
                    position_rect,
                    tex_coord_rect,
                    color,
                    texture_id: self.ui.font_atlas_texture_id,
                });

                parent.draw_range.end += 1;
                if extend_content_rect {
                    parent.inline_content_rect = Some(
                        parent
                            .inline_content_rect
                            .map(|r| r.extend_by_rect(position_rect))
                            .unwrap_or(position_rect),
                    );
                }

                position_x += glyph_advance_width;
            }

            position_y += line_metrics.new_line_size;
        }
    }
}
