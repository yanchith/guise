use core::alloc::Allocator;
use core::fmt::Debug;

use crate::core::{Ctrl, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Vec2};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

const FLAGS: CtrlFlags =
    CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE;

const ACTIVITY_NONE: u8 = 0;
const ACTIVITY_MOVE: u8 = 1;
const ACTIVITY_RESIZE: u8 = 2;

// TODO(yan): Decide if we want an RAII thing, or an explicit end for widgets
pub fn begin_window<'f, X, Y, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    x: X,
    y: Y,
    width: W,
    height: H,
) -> Ctrl<'f, A>
where
    X: TryInto<Size>,
    Y: TryInto<Size>,
    W: TryInto<Size>,
    H: TryInto<Size>,
    <X as TryInto<Size>>::Error: Debug,
    <Y as TryInto<Size>>::Error: Debug,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    Window::new(id, x, y, width, height).begin(frame)
}

pub fn end_window<A: Allocator + Clone>(frame: &mut Frame<A>) {
    frame.pop_ctrl();
}

pub struct Window<'a> {
    id: u32,
    x: Size,
    y: Size,
    width: Size,
    height: Size,

    movable: bool,
    resizable: bool,
    open_on_top: bool,

    layout: Layout,

    theme: &'a Theme,
}

impl<'a> Window<'a> {
    pub fn new<X, Y, W, H>(id: u32, x: X, y: Y, width: W, height: H) -> Self
    where
        X: TryInto<Size>,
        Y: TryInto<Size>,
        W: TryInto<Size>,
        H: TryInto<Size>,
        <X as TryInto<Size>>::Error: Debug,
        <Y as TryInto<Size>>::Error: Debug,
        <W as TryInto<Size>>::Error: Debug,
        <H as TryInto<Size>>::Error: Debug,
    {
        let x = x.try_into().unwrap();
        let y = y.try_into().unwrap();
        let width = width.try_into().unwrap();
        let height = height.try_into().unwrap();

        Self {
            id,
            x,
            y,
            width,
            height,

            movable: true,
            resizable: true,
            open_on_top: true,

            layout: Layout::Vertical,

            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_movable(&mut self, movable: bool) -> &mut Self {
        self.movable = movable;
        self
    }

    pub fn set_resizable(&mut self, resizable: bool) -> &mut Self {
        self.resizable = resizable;
        self
    }

    pub fn set_open_on_top(&mut self, open_on_top: bool) -> &mut Self {
        self.open_on_top = open_on_top;
        self
    }

    pub fn set_layout(&mut self, layout: Layout) -> &mut Self {
        self.layout = layout;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn begin<'f, A: Allocator + Clone>(&self, frame: &'f mut Frame<A>) -> Ctrl<'f, A> {
        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();
        let lmb_pressed = frame.inputs_pressed() == Inputs::MB_LEFT;
        let lmb_released = frame.inputs_released() == Inputs::MB_LEFT;

        let mut ctrl = frame.push_ctrl(self.id);
        let hovered = ctrl.hovered();

        let state = ctrl.state();
        let (x, y, mut width, mut height, activity, initialized) = if initialized(state) {
            let (x, y) = if self.movable {
                (x(state), y(state))
            } else {
                (self.x.resolve(parent_size.x), self.y.resolve(parent_size.y))
            };

            let (width, height) = if self.resizable {
                (width(state), height(state))
            } else {
                (
                    self.width.resolve(parent_size.x),
                    self.height.resolve(parent_size.y),
                )
            };

            let activity = match (activity(state), self.movable, self.resizable) {
                (ACTIVITY_MOVE, false, _) => ACTIVITY_NONE,
                (ACTIVITY_RESIZE, _, false) => ACTIVITY_NONE,
                (activity, _, _) => activity,
            };

            (x, y, width, height, activity, true)
        } else {
            (
                self.x.resolve(parent_size.x),
                self.y.resolve(parent_size.y),
                self.width.resolve(parent_size.x),
                self.height.resolve(parent_size.y),
                ACTIVITY_NONE,
                false,
            )
        };

        ctrl.set_flags(FLAGS);
        ctrl.set_layout(self.layout);
        ctrl.set_rect(Rect::new(x, y, width, height));
        ctrl.set_padding(self.theme.window_padding);
        ctrl.set_border(self.theme.window_border);
        ctrl.set_margin(0.0);

        let resize_handle_dimension = self.theme.window_padding + self.theme.window_border;
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

        let state = ctrl.state_mut();
        set_x(state, x);
        set_y(state, y);
        set_width(state, width);
        set_height(state, height);
        set_initialized(state, true);

        if !self.movable && activity == ACTIVITY_MOVE {
            set_activity(state, ACTIVITY_NONE);
        }
        if !self.resizable && activity == ACTIVITY_RESIZE {
            set_activity(state, ACTIVITY_NONE);
        }

        if activity == ACTIVITY_RESIZE {
            if lmb_released {
                set_activity(state, ACTIVITY_NONE);
            } else {
                let activity_start_x = activity_start_x(state);
                let activity_start_y = activity_start_y(state);
                let activity_start_size = Vec2::new(activity_start_x, activity_start_y);

                let activity_start_cursor_x = activity_start_cursor_x(state);
                let activity_start_cursor_y = activity_start_cursor_y(state);
                let activity_start_cursor_position =
                    Vec2::new(activity_start_cursor_x, activity_start_cursor_y);

                let size = activity_start_size + cursor_position - activity_start_cursor_position;
                let size_clamped = size.max(Vec2::ZERO);

                width = size_clamped.x;
                height = size_clamped.y;

                set_width(state, width);
                set_height(state, height);

                // Set rect again with updated data to reduce latency
                ctrl.set_rect(Rect::new(x, y, width, height));
            }
        } else if self.resizable && hovered && resize_handle_hovered && lmb_pressed {
            set_activity(state, ACTIVITY_RESIZE);
            set_activity_start_x(state, width);
            set_activity_start_y(state, height);
            set_activity_start_cursor_x(state, cursor_position.x);
            set_activity_start_cursor_y(state, cursor_position.y);
        } else if activity == ACTIVITY_MOVE {
            if lmb_released {
                set_activity(state, ACTIVITY_NONE);
            } else {
                let activity_start_x = activity_start_x(state);
                let activity_start_y = activity_start_y(state);
                let activity_start_position = Vec2::new(activity_start_x, activity_start_y);

                let activity_start_cursor_x = activity_start_cursor_x(state);
                let activity_start_cursor_y = activity_start_cursor_y(state);
                let activity_start_cursor_position =
                    Vec2::new(activity_start_cursor_x, activity_start_cursor_y);

                let position =
                    activity_start_position + cursor_position - activity_start_cursor_position;

                set_x(state, position.x);
                set_y(state, position.y);

                // Set rect again with updated data to reduce latency
                ctrl.set_rect(Rect::new(position.x, position.y, width, height));
            }
        } else if self.movable && hovered && lmb_pressed {
            set_activity(state, ACTIVITY_MOVE);
            set_activity_start_x(state, x);
            set_activity_start_y(state, y);
            set_activity_start_cursor_x(state, cursor_position.x);
            set_activity_start_cursor_y(state, cursor_position.y);
        }

        if hovered && lmb_pressed || self.open_on_top && !initialized {
            ctrl.set_active(true);
        }

        let (background_color, border_color, resize_handle_color) = match (
            hovered,
            resize_handle_hovered || activity == ACTIVITY_RESIZE,
        ) {
            (false, _) => (
                self.theme.window_background_color,
                self.theme.window_border_color,
                self.theme.window_border_color,
            ),
            (true, false) => (
                self.theme.window_background_color_hovered,
                self.theme.window_border_color_hovered,
                self.theme.window_border_color_hovered,
            ),
            (true, true) => (
                self.theme.window_background_color_hovered,
                self.theme.window_border_color_hovered,
                0xffffffff,
            ),
        };

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(border_color);
        ctrl.set_draw_self_background_color(background_color);

        if self.resizable {
            ctrl.draw_rect(
                false,
                Rect::new(
                    width - resize_handle_dimension,
                    height - resize_handle_dimension,
                    resize_handle_dimension,
                    resize_handle_dimension,
                ),
                Rect::ZERO,
                resize_handle_color,
                0,
            );
        }

        ctrl
    }

    pub fn end<A: Allocator + Clone>(&self, frame: &mut Frame<A>) {
        frame.pop_ctrl();
    }
}

fn x(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[0], state[1], state[2], state[3]])
}

fn y(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[4], state[5], state[6], state[7]])
}

fn width(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[8], state[9], state[10], state[11]])
}

fn height(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[12], state[13], state[14], state[15]])
}

fn activity_start_cursor_x(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[16], state[17], state[18], state[19]])
}

fn activity_start_cursor_y(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[20], state[21], state[22], state[23]])
}

fn activity_start_x(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[24], state[25], state[26], state[27]])
}

fn activity_start_y(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[28], state[29], state[30], state[31]])
}

fn activity(state: &CtrlState) -> u8 {
    state[32]
}

fn initialized(state: &CtrlState) -> bool {
    state[33] == 1
}

fn set_x(state: &mut CtrlState, x: f32) {
    let bytes = x.to_le_bytes();
    state[0] = bytes[0];
    state[1] = bytes[1];
    state[2] = bytes[2];
    state[3] = bytes[3];
}

fn set_y(state: &mut CtrlState, y: f32) {
    let bytes = y.to_le_bytes();
    state[4] = bytes[0];
    state[5] = bytes[1];
    state[6] = bytes[2];
    state[7] = bytes[3];
}

fn set_width(state: &mut CtrlState, width: f32) {
    let bytes = width.to_le_bytes();
    state[8] = bytes[0];
    state[9] = bytes[1];
    state[10] = bytes[2];
    state[11] = bytes[3];
}

fn set_height(state: &mut CtrlState, height: f32) {
    let bytes = height.to_le_bytes();
    state[12] = bytes[0];
    state[13] = bytes[1];
    state[14] = bytes[2];
    state[15] = bytes[3];
}

fn set_activity_start_cursor_x(state: &mut CtrlState, activity_start_cursor_x: f32) {
    let bytes = activity_start_cursor_x.to_le_bytes();
    state[16] = bytes[0];
    state[17] = bytes[1];
    state[18] = bytes[2];
    state[19] = bytes[3];
}

fn set_activity_start_cursor_y(state: &mut CtrlState, activity_start_cursor_y: f32) {
    let bytes = activity_start_cursor_y.to_le_bytes();
    state[20] = bytes[0];
    state[21] = bytes[1];
    state[22] = bytes[2];
    state[23] = bytes[3];
}

fn set_activity_start_x(state: &mut CtrlState, activity_start_x: f32) {
    let bytes = activity_start_x.to_le_bytes();
    state[24] = bytes[0];
    state[25] = bytes[1];
    state[26] = bytes[2];
    state[27] = bytes[3];
}

fn set_activity_start_y(state: &mut CtrlState, activity_start_y: f32) {
    let bytes = activity_start_y.to_le_bytes();
    state[28] = bytes[0];
    state[29] = bytes[1];
    state[30] = bytes[2];
    state[31] = bytes[3];
}

fn set_activity(state: &mut CtrlState, activity: u8) {
    state[32] = activity
}

fn set_initialized(state: &mut CtrlState, initialized: bool) {
    state[33] = if initialized { 1 } else { 0 }
}
