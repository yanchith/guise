use core::alloc::Allocator;
use core::fmt::Write;

use arrayvec::ArrayString;

use crate::core::{Align, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

// TODO(yan): Do DragFloat2, DragFloat3, DragFloat4.

pub fn drag_float<A, TA>(frame: &mut Frame<A, TA>, id: u32, value: &mut f32, speed: f32) -> bool
where
    A: Allocator + Clone,
    TA: Allocator,
{
    DragFloat::new(id, value).set_speed(speed).show(frame)
}

pub fn drag_float_ex<A, TA>(
    frame: &mut Frame<A, TA>,
    id: u32,
    value: &mut f32,
    speed: f32,
    min: f32,
    max: f32,
) -> bool
where
    A: Allocator + Clone,
    TA: Allocator,
{
    DragFloat::new(id, value)
        .set_speed(speed)
        .set_min(min)
        .set_max(max)
        .show(frame)
}

pub struct DragFloat<'a> {
    id: u32,
    value: &'a mut f32,
    theme: &'a Theme,

    speed: f32,
    min: f32,
    max: f32,
}

impl<'a> DragFloat<'a> {
    pub fn new(id: u32, value: &'a mut f32) -> Self {
        Self {
            id,
            value,
            theme: &Theme::DEFAULT,

            speed: 1.0,
            min: f32::MIN,
            max: f32::MAX,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn set_speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn set_min(&mut self, min: f32) -> &mut Self {
        self.min = min;
        self
    }

    pub fn set_max(&mut self, max: f32) -> &mut Self {
        self.max = max;
        self
    }

    pub fn show<A, TA>(&mut self, frame: &mut Frame<A, TA>) -> bool
    where
        A: Allocator + Clone,
        TA: Allocator,
    {
        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();
        let inputs_pressed = frame.inputs_pressed();
        let inputs_released = frame.inputs_released();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.drag_float_margin);

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_HOVER);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.drag_float_height));
        ctrl.set_padding(self.theme.drag_float_padding);
        ctrl.set_border(self.theme.drag_float_border);
        ctrl.set_margin(self.theme.drag_float_margin);

        let hovered = ctrl.hovered();
        let active = ctrl.active();
        let state = ctrl.state();

        let (active, changed) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MOUSE_BUTTON_LEFT {
                ctrl.set_active(false);
                false
            } else {
                true
            };

            let old_value = *self.value;
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            *self.value = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MOUSE_BUTTON_LEFT {
            ctrl.set_active(true);
            let state = ctrl.state_mut();
            set_value(state, *self.value);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            ctrl.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(border_color);
        ctrl.set_draw_self_background_color(background_color);

        let mut text: ArrayString<256> = ArrayString::new();
        let _ = write!(text, "{:.3}", *self.value);
        ctrl.draw_text_ex(
            true,
            None,
            0.0,
            &text,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        changed
    }
}

fn value(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[0], state[1], state[2], state[3]])
}

fn x(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[4], state[5], state[6], state[7]])
}

fn set_value(state: &mut CtrlState, value: f32) {
    let bytes = value.to_le_bytes();
    state[0] = bytes[0];
    state[1] = bytes[1];
    state[2] = bytes[2];
    state[3] = bytes[3];
}

fn set_x(state: &mut CtrlState, x: f32) {
    let bytes = x.to_le_bytes();
    state[4] = bytes[0];
    state[5] = bytes[1];
    state[6] = bytes[2];
    state[7] = bytes[3];
}
