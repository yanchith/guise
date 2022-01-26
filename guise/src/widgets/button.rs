use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Vec2, Wrap};
use crate::widgets::theme::Theme;

pub fn button(frame: &mut Frame, id: u32, label: &str) -> bool {
    let theme = &Theme::DEFAULT;
    Button::new(id, theme, label).show(frame)
}

pub struct Button<'a> {
    id: u32,
    theme: &'a Theme,
    label: &'a str,

    x: f32,
    y: f32,
}

impl<'a> Button<'a> {
    pub fn new(id: u32, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            id,
            theme,
            label,

            x: 0.0,
            y: 0.0,
        }
    }

    pub fn show(&self, frame: &mut Frame) -> bool {
        let lmb_pressed = frame.window_inputs_pressed() == Inputs::MOUSE_BUTTON_LEFT;
        let lmb_released = frame.window_inputs_released() == Inputs::MOUSE_BUTTON_LEFT;

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(
            self.x,
            self.y,
            self.theme.button_width,
            self.theme.button_height,
        ));
        ctrl.set_padding(0.0);
        ctrl.set_border(self.theme.button_border);
        ctrl.set_margin(self.theme.button_margin);

        let hovered = ctrl.hovered();
        let active = ctrl.active();

        let (active, changed) = if active && lmb_released {
            ctrl.set_active(false);
            if hovered {
                // Make the control inactive once again after release, as the
                // platform may not be running us on every frame, but only for
                // new events. Also better latency this way.
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

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.button_text_color,
                self.theme.button_background_color,
                self.theme.button_border_color,
            ),
            (true, false) => (
                self.theme.button_text_color_hovered,
                self.theme.button_background_color_hovered,
                self.theme.button_border_color_hovered,
            ),
            (_, true) => (
                self.theme.button_text_color_active,
                self.theme.button_background_color_active,
                self.theme.button_border_color_active,
            ),
        };

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(border_color);
        ctrl.set_draw_self_background_color(background_color);
        ctrl.draw_text(
            false,
            Vec2::ZERO,
            self.label,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        changed
    }
}
