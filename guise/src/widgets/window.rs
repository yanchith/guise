use core::alloc::Allocator;
use core::fmt::Debug;
use core::mem;

use crate::core::{Ctrl, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Vec2};
use crate::widgets::size::{Position, Size};
use crate::widgets::theme::Theme;

const FLAGS: CtrlFlags =
    CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE;

const ACTIVITY_NONE: u8 = 0;
const ACTIVITY_MOVE: u8 = 1;
const ACTIVITY_RESIZE: u8 = 2;

const DEFAULT_OPTIONS: WindowOptions = WindowOptions {
    movable: true,
    resizable: true,
    open_on_top: true,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowOptions {
    pub movable: bool,
    pub resizable: bool,
    pub open_on_top: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        DEFAULT_OPTIONS
    }
}

// TODO(yan): Make this actually return None when the window is collapsed,
// minimized, or something.

#[inline]
pub fn begin_window<'f, X, Y, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    x: X,
    y: Y,
    width: W,
    height: H,
) -> Option<(Window, Ctrl<'f, A>)>
where
    X: TryInto<Position>,
    Y: TryInto<Position>,
    W: TryInto<Size>,
    H: TryInto<Size>,
    <X as TryInto<Position>>::Error: Debug,
    <Y as TryInto<Position>>::Error: Debug,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let x = x.try_into().unwrap();
    let y = y.try_into().unwrap();
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_window_and_pay_bills(
        frame,
        id,
        x,
        y,
        width,
        height,
        Layout::Vertical,
        &DEFAULT_OPTIONS,
        &Theme::DEFAULT,
    );

    Some((Window(false), ctrl))
}

#[inline]
pub fn begin_window_with_layout<'f, X, Y, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    x: X,
    y: Y,
    width: W,
    height: H,
    layout: Layout,
) -> Option<(Window, Ctrl<'f, A>)>
where
    X: TryInto<Position>,
    Y: TryInto<Position>,
    W: TryInto<Size>,
    H: TryInto<Size>,
    <X as TryInto<Position>>::Error: Debug,
    <Y as TryInto<Position>>::Error: Debug,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let x = x.try_into().unwrap();
    let y = y.try_into().unwrap();
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_window_and_pay_bills(
        frame,
        id,
        x,
        y,
        width,
        height,
        layout,
        &DEFAULT_OPTIONS,
        &Theme::DEFAULT,
    );

    Some((Window(false), ctrl))
}

#[inline]
pub fn begin_window_with_layout_options<'f, X, Y, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    x: X,
    y: Y,
    width: W,
    height: H,
    layout: Layout,
    options: &WindowOptions,
) -> Option<(Window, Ctrl<'f, A>)>
where
    X: TryInto<Position>,
    Y: TryInto<Position>,
    W: TryInto<Size>,
    H: TryInto<Size>,
    <X as TryInto<Position>>::Error: Debug,
    <Y as TryInto<Position>>::Error: Debug,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let x = x.try_into().unwrap();
    let y = y.try_into().unwrap();
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_window_and_pay_bills(
        frame,
        id,
        x,
        y,
        width,
        height,
        layout,
        options,
        &Theme::DEFAULT,
    );

    Some((Window(false), ctrl))
}

#[inline]
pub fn begin_window_with_layout_options_theme<'f, X, Y, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    x: X,
    y: Y,
    width: W,
    height: H,
    layout: Layout,
    options: &WindowOptions,
    theme: &Theme,
) -> Option<(Window, Ctrl<'f, A>)>
where
    X: TryInto<Position>,
    Y: TryInto<Position>,
    W: TryInto<Size>,
    H: TryInto<Size>,
    <X as TryInto<Position>>::Error: Debug,
    <Y as TryInto<Position>>::Error: Debug,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let x = x.try_into().unwrap();
    let y = y.try_into().unwrap();
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_window_and_pay_bills(frame, id, x, y, width, height, layout, options, theme);

    Some((Window(false), ctrl))
}

pub struct Window(bool);

impl Window {
    pub fn end<A: Allocator + Clone>(mut self, frame: &mut Frame<A>) {
        assert!(!self.0);

        frame.pop_ctrl();
        self.0 = true;
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        debug_assert!(self.0)
    }
}

fn do_window_and_pay_bills<'f, A: Allocator + Clone>(
    frame: &'f mut Frame<A>,
    id: u32,
    x: Position,
    y: Position,
    width: Size,
    height: Size,
    layout: Layout,
    options: &WindowOptions,
    theme: &Theme,
) -> Ctrl<'f, A> {
    let texture_id = frame.font_atlas_texture_id();
    let parent_size = frame.ctrl_inner_size();
    let cursor_position = frame.cursor_position();
    let lmb_pressed = frame.inputs_pressed() == Inputs::MB_LEFT;
    let lmb_released = frame.inputs_released() == Inputs::MB_LEFT;

    let mut ctrl = frame.push_ctrl(id);
    let hovered = ctrl.is_hovered();

    let state = cast_state(ctrl.state());
    let (x, y, mut width, mut height, activity, initialized) = if state.initialized == 1 {
        let (x, y) = if options.movable {
            (state.x, state.y)
        } else {
            (x.resolve(parent_size.x), y.resolve(parent_size.y))
        };

        let (width, height) = if options.resizable {
            (state.width, state.height)
        } else {
            (width.resolve(parent_size.x), height.resolve(parent_size.y))
        };

        let activity = match (state.activity, options.movable, options.resizable) {
            (ACTIVITY_MOVE, false, _) => ACTIVITY_NONE,
            (ACTIVITY_RESIZE, _, false) => ACTIVITY_NONE,
            (activity, _, _) => activity,
        };

        (x, y, width, height, activity, true)
    } else {
        (
            x.resolve(parent_size.x),
            y.resolve(parent_size.y),
            width.resolve(parent_size.x),
            height.resolve(parent_size.y),
            ACTIVITY_NONE,
            false,
        )
    };

    ctrl.set_flags(FLAGS);
    ctrl.set_layout(layout);
    ctrl.set_rect(Rect::new(x, y, width, height));
    ctrl.set_padding(theme.window_padding);
    ctrl.set_border(theme.window_border);
    ctrl.set_margin(0.0);

    let resize_handle_dimension = theme.window_padding + theme.window_border;
    let resize_handle_hovered = {
        let position = ctrl.absolute_position();
        let rect = Rect::new(
            position.x + width - resize_handle_dimension,
            position.y + height - resize_handle_dimension,
            resize_handle_dimension,
            resize_handle_dimension,
        );
        rect.contains_point(cursor_position)
    };

    let state = cast_state_mut(ctrl.state_mut());
    state.x = x;
    state.y = y;
    state.width = width;
    state.height = height;
    state.initialized = 1;

    if !options.movable && activity == ACTIVITY_MOVE {
        state.activity = ACTIVITY_NONE;
    }
    if !options.resizable && activity == ACTIVITY_RESIZE {
        state.activity = ACTIVITY_NONE;
    }

    if activity == ACTIVITY_RESIZE {
        if lmb_released {
            state.activity = ACTIVITY_NONE;
        } else {
            let activity_start_x = state.activity_start_x;
            let activity_start_y = state.activity_start_y;
            let activity_start_size = Vec2::new(activity_start_x, activity_start_y);

            let activity_start_cursor_x = state.activity_start_cursor_x;
            let activity_start_cursor_y = state.activity_start_cursor_y;
            let activity_start_cursor_position =
                Vec2::new(activity_start_cursor_x, activity_start_cursor_y);

            let size = activity_start_size + cursor_position - activity_start_cursor_position;
            let size_clamped = size.max(Vec2::ZERO);

            width = size_clamped.x;
            height = size_clamped.y;

            state.width = width;
            state.height = height;

            // Set rect again with updated data to reduce latency
            ctrl.set_rect(Rect::new(x, y, width, height));
        }
    } else if options.resizable && hovered && resize_handle_hovered && lmb_pressed {
        state.activity = ACTIVITY_RESIZE;
        state.activity_start_x = width;
        state.activity_start_y = height;
        state.activity_start_cursor_x = cursor_position.x;
        state.activity_start_cursor_y = cursor_position.y;
    } else if activity == ACTIVITY_MOVE {
        if lmb_released {
            state.activity = ACTIVITY_NONE;
        } else {
            let activity_start_x = state.activity_start_x;
            let activity_start_y = state.activity_start_y;
            let activity_start_position = Vec2::new(activity_start_x, activity_start_y);

            let activity_start_cursor_x = state.activity_start_cursor_x;
            let activity_start_cursor_y = state.activity_start_cursor_y;
            let activity_start_cursor_position =
                Vec2::new(activity_start_cursor_x, activity_start_cursor_y);

            let position =
                activity_start_position + cursor_position - activity_start_cursor_position;

            state.x = position.x;
            state.y = position.y;

            // Set rect again with updated data to reduce latency
            ctrl.set_rect(Rect::new(position.x, position.y, width, height));
        }
    } else if options.movable && hovered && lmb_pressed {
        state.activity = ACTIVITY_MOVE;
        state.activity_start_x = x;
        state.activity_start_y = y;
        state.activity_start_cursor_x = cursor_position.x;
        state.activity_start_cursor_y = cursor_position.y;
    }

    if hovered && lmb_pressed || options.open_on_top && !initialized {
        ctrl.set_active(true);
    }

    let (background_color, border_color, resize_handle_color) = match (
        hovered,
        resize_handle_hovered || activity == ACTIVITY_RESIZE,
    ) {
        (false, _) => (
            theme.window_background_color,
            theme.window_border_color,
            theme.window_border_color,
        ),
        (true, false) => (
            theme.window_background_color_hovered,
            theme.window_border_color_hovered,
            theme.window_border_color_hovered,
        ),
        (true, true) => (
            theme.window_background_color_hovered,
            theme.window_border_color_hovered,
            0xffffffff,
        ),
    };

    ctrl.set_draw_self(true);
    ctrl.set_draw_self_border_color(border_color);
    ctrl.set_draw_self_background_color(background_color);

    if options.resizable {
        let offset_x = ctrl.scroll_offset_x();
        let offset_y = ctrl.scroll_offset_y();

        ctrl.draw_rect(
            Rect::new(
                width - resize_handle_dimension + offset_x,
                height - resize_handle_dimension + offset_y,
                resize_handle_dimension,
                resize_handle_dimension,
            ),
            Rect::ZERO,
            resize_handle_color,
            texture_id,
        );
    }

    ctrl
}

#[repr(C)]
#[derive(Clone, Copy)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
struct State {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    activity_start_cursor_x: f32,
    activity_start_cursor_y: f32,
    activity_start_x: f32,
    activity_start_y: f32,
    activity: u8,
    initialized: u8,
    _pad0: u8,
    _pad1: u8,
}

fn cast_state(state: &CtrlState) -> &State {
    bytemuck::from_bytes(&state[..mem::size_of::<State>()])
}

fn cast_state_mut(state: &mut CtrlState) -> &mut State {
    bytemuck::from_bytes_mut(&mut state[..mem::size_of::<State>()])
}
