use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Vec2, Wrap};
use crate::widgets::theme::Theme;

pub fn checkbox(frame: &mut Frame, id: u32, value: &mut bool, label: &str) -> bool {
    let theme = &Theme::DEFAULT;
    Checkbox::new(id, theme, value, label).show(frame)
}

pub struct Checkbox<'a> {
    id: u32,
    theme: &'a Theme,
    value: &'a mut bool,
    label: &'a str,

    x: f32,
    y: f32,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: u32, theme: &'a Theme, value: &'a mut bool, label: &'a str) -> Self {
        Self {
            id,
            theme,
            value,
            label,

            x: 0.0,
            y: 0.0,
        }
    }

    pub fn show(&mut self, frame: &mut Frame) -> bool {
        let lmb_pressed = frame.window_inputs_pressed() == Inputs::MOUSE_BUTTON_LEFT;
        let lmb_released = frame.window_inputs_released() == Inputs::MOUSE_BUTTON_LEFT;

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(
            self.x,
            self.y,
            self.theme.checkbox_width,
            self.theme.checkbox_height,
        ));
        ctrl.set_padding(0.0);
        ctrl.set_border(self.theme.checkbox_border);
        ctrl.set_margin(self.theme.checkbox_margin);

        let hovered = ctrl.hovered();
        let active = ctrl.active();

        let (active, changed) = if active && lmb_released {
            ctrl.set_active(false);
            if hovered {
                // Make the control inactive once again after release, as the
                // platform may not be running us on every frame, but only for
                // new events. Also better latency this way.
                *self.value = !*self.value;
                (false, true)
            } else {
                (false, false)
            }
        } else if hovered && lmb_pressed {
            ctrl.set_active(true);
            (true, false)
        } else {
            (active, false)
        };

        let (handle_color, text_color) = match (hovered, active) {
            (false, false) => (
                self.theme.checkbox_handle_color,
                self.theme.checkbox_text_color,
            ),
            (true, false) => (
                self.theme.checkbox_handle_color_hovered,
                self.theme.checkbox_text_color_hovered,
            ),
            (_, true) => (
                self.theme.checkbox_handle_color_active,
                self.theme.checkbox_text_color_active,
            ),
        };

        ctrl.set_draw_self(false);
        ctrl.draw_rect(
            false,
            Rect::new(5.0, 7.5, 20.0, 20.0),
            Rect::ZERO,
            handle_color,
            0,
        );

        if *self.value {
            ctrl.draw_rect(
                false,
                Rect::new(10.0, 12.5, 10.0, 10.0),
                Rect::ZERO,
                0xffffffff,
                0,
            );
        }

        ctrl.draw_text(
            false,
            Vec2::new(40.0, 0.0),
            self.label,
            Align::Start,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        changed
    }
}
