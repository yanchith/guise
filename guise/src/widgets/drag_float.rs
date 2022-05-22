use core::alloc::Allocator;
use core::fmt::Write;

use arrayvec::ArrayString;

use crate::core::{Align, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

// TODO(yan): Do DragFloat2, DragFloat3, DragFloat4.

const LABEL_WIDTH_RATIO: f32 = 0.4;
const LABEL_SPACING: f32 = 5.0;

pub fn drag_float<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut f32,
    label: &str,
) -> bool {
    DragFloat::new(id, value, label).show(frame)
}

pub struct DragFloat<'a> {
    id: u32,
    value: &'a mut f32,
    label: &'a str,

    speed: f32,
    min: f32,
    max: f32,
    display_precision: u16,

    theme: &'a Theme,
}

impl<'a> DragFloat<'a> {
    pub fn new(id: u32, value: &'a mut f32, label: &'a str) -> Self {
        Self {
            id,
            value,
            label,

            speed: 1.0,
            min: f32::MIN,
            max: f32::MAX,
            display_precision: 3,

            theme: &Theme::DEFAULT,
        }
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

    pub fn set_display_precision(&mut self, display_precision: u16) -> &mut Self {
        self.display_precision = display_precision;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&mut self, frame: &mut Frame<A>) -> bool {
        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();
        let inputs_pressed = frame.inputs_pressed();
        let inputs_released = frame.inputs_released();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.drag_float_margin);
        let label_width = LABEL_WIDTH_RATIO * width;
        let inner_width = f32::max(0.0, width - label_width - LABEL_SPACING);

        let mut outer_ctrl = frame.push_ctrl(self.id);
        outer_ctrl.set_flags(CtrlFlags::NONE);
        outer_ctrl.set_layout(Layout::Horizontal);
        outer_ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.drag_float_height));
        outer_ctrl.set_padding(0.0);
        outer_ctrl.set_border(0.0);
        outer_ctrl.set_margin(self.theme.drag_float_margin);

        outer_ctrl.set_draw_self(false);
        outer_ctrl.draw_text(
            true,
            Some(Rect::new(
                0.0,
                0.0,
                label_width,
                self.theme.drag_float_height,
            )),
            0.0,
            self.label,
            Align::Start,
            Align::Center,
            Wrap::Word,
            self.theme.drag_float_text_color,
        );

        let mut inner_ctrl = frame.push_ctrl(0);
        inner_ctrl.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl.set_layout(Layout::Vertical);
        inner_ctrl.set_rect(Rect::new(
            label_width + LABEL_SPACING,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl.set_padding(0.0);
        inner_ctrl.set_border(self.theme.drag_float_border);
        inner_ctrl.set_margin(0.0);

        let hovered = inner_ctrl.hovered();
        let active = inner_ctrl.active();
        let state = inner_ctrl.state();

        let (active, changed) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl.set_active(false);
                false
            } else {
                true
            };

            let old_value = *self.value;
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            *self.value = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl.set_active(true);
            let state = inner_ctrl.state_mut();
            set_value(state, *self.value);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl.request_want_capture_keyboard();
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

        inner_ctrl.set_draw_self(true);
        inner_ctrl.set_draw_self_border_color(border_color);
        inner_ctrl.set_draw_self_background_color(background_color);

        let mut text: ArrayString<256> = ArrayString::new();
        let _ = write!(
            text,
            "{:.1$}",
            *self.value,
            usize::from(self.display_precision)
        );
        inner_ctrl.draw_text(
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
