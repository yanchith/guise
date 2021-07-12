use core::convert::TryInto;
use core::fmt::Debug;

use crate::core::{Ctrl, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Vec2};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

const ACTIVITY_NONE: u8 = 0;
const ACTIVITY_MOVE: u8 = 1;
const ACTIVITY_RESIZE: u8 = 2;

// TODO(yan): Decide if we want an RAII thing, or an explicit end for widgets
pub fn begin_window<'f, X, Y, W, H>(
    frame: &'f mut Frame,
    id: u32,
    theme: &Theme,
    layout: Layout,
    x: X,
    y: Y,
    width: W,
    height: H,
) -> Option<Ctrl<'f>>
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
    Window::new(id, theme, layout, x, y, width, height).begin(frame)
}

pub fn end_window(frame: &mut Frame) {
    frame.pop_ctrl();
}

pub struct Window<'a> {
    id: u32,
    theme: &'a Theme,
    layout: Layout,

    x: Size,
    y: Size,
    width: Size,
    height: Size,
}

impl<'a> Window<'a> {
    pub fn new<X, Y, W, H>(
        id: u32,
        theme: &'a Theme,
        layout: Layout,
        x: X,
        y: Y,
        width: W,
        height: H,
    ) -> Self
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
            theme,
            layout,
            x,
            y,
            width,
            height,
        }
    }

    pub fn begin<'f>(&self, frame: &'f mut Frame) -> Option<Ctrl<'f>> {
        let parent_extents = frame.ctrl_inner_extents();
        let cursor_position = frame.window_cursor_position();
        let lmb_pressed = frame.window_inputs_pressed() == Inputs::MOUSE_BUTTON_LEFT;
        let lmb_released = frame.window_inputs_released() == Inputs::MOUSE_BUTTON_LEFT;

        let mut ctrl = frame.push_ctrl(self.id);
        let hovered = ctrl.hovered();

        let state = ctrl.state();
        let (x, y, mut width, mut height, activity) = if initialized(state) {
            (
                x(state),
                y(state),
                width(state),
                height(state),
                activity(state),
            )
        } else {
            (
                self.x.resolve(parent_extents.x),
                self.y.resolve(parent_extents.y),
                self.width.resolve(parent_extents.x),
                self.height.resolve(parent_extents.y),
                ACTIVITY_NONE,
            )
        };

        ctrl.set_flags(
            CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE,
        );
        ctrl.set_layout(self.layout);
        ctrl.set_rect(Rect::new(x, y, width, height));
        ctrl.set_padding(self.theme.window_padding);
        ctrl.set_border(self.theme.window_border);
        ctrl.set_margin(0.0);

        let resize_handle_dimension = self.theme.window_padding + self.theme.window_border;
        let resize_handle_hovered = {
            let position = ctrl.window_position();
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

        if activity == ACTIVITY_RESIZE {
            if lmb_released {
                set_activity(state, ACTIVITY_NONE);
            } else {
                let activity_start_x = activity_start_x(state);
                let activity_start_y = activity_start_y(state);
                let activity_start_extents = Vec2::new(activity_start_x, activity_start_y);

                let activity_start_cursor_x = activity_start_cursor_x(state);
                let activity_start_cursor_y = activity_start_cursor_y(state);
                let activity_start_cursor_position =
                    Vec2::new(activity_start_cursor_x, activity_start_cursor_y);

                let extents =
                    activity_start_extents + cursor_position - activity_start_cursor_position;
                let extents_clamped = extents.max(Vec2::ZERO);

                width = extents_clamped.x;
                height = extents_clamped.y;

                set_width(state, width);
                set_height(state, height);

                // Set rect again with updated data to reduce latency
                ctrl.set_rect(Rect::new(x, y, width, height));
            }
        } else if hovered && resize_handle_hovered && lmb_pressed {
            set_activity(state, ACTIVITY_RESIZE);
            set_activity_start_x(state, width);
            set_activity_start_y(state, height);
            set_activity_start_cursor_x(state, cursor_position.x);
            set_activity_start_cursor_y(state, cursor_position.y);

            ctrl.set_active(true);
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
        } else if hovered && lmb_pressed {
            set_activity(state, ACTIVITY_MOVE);
            set_activity_start_x(state, x);
            set_activity_start_y(state, y);
            set_activity_start_cursor_x(state, cursor_position.x);
            set_activity_start_cursor_y(state, cursor_position.y);

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

        // TODO(yan): Would be great for simplicity if draw_rect used the same
        // coordinate system as we use for positioning controls. Currently we
        // position using border-box, but draw_rect uses content-box. We'd still
        // clip by at border though. Beware text!
        ctrl.draw_rect(
            false,
            Rect::new(
                width
                    - resize_handle_dimension
                    - self.theme.window_border
                    - self.theme.window_padding,
                height
                    - resize_handle_dimension
                    - self.theme.window_border
                    - self.theme.window_padding,
                resize_handle_dimension,
                resize_handle_dimension,
            ),
            Rect::ZERO,
            resize_handle_color,
            0,
        );

        // TODO(yan): Will this ever be None?
        Some(ctrl)
    }

    pub fn end(&self, frame: &mut Frame) {
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
