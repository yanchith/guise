use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

pub fn checkbox<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut bool,
    label: &str,
) -> bool {
    Checkbox::new(id, value, label).show(frame)
}

pub struct Checkbox<'a> {
    id: u32,
    value: &'a mut bool,
    label: &'a str,
    theme: &'a Theme,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: u32, value: &'a mut bool, label: &'a str) -> Self {
        Self {
            id,
            value,
            label,
            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&mut self, frame: &mut Frame<A>) -> bool {
        let texture_id = frame.font_atlas_texture_id();
        let parent_size = frame.ctrl_inner_size();
        let lmb_pressed = frame.inputs_pressed() == Inputs::MB_LEFT;
        let lmb_released = frame.inputs_released() == Inputs::MB_LEFT;

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.checkbox_margin);

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.checkbox_height));
        ctrl.set_padding(0.0);
        ctrl.set_border(self.theme.checkbox_border);
        ctrl.set_margin(self.theme.checkbox_margin);

        let hovered = ctrl.is_hovered();
        let active = ctrl.is_active();

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

        const CHECKBOX_LEFT_PADDING: f32 = 5.0;
        const CHECKBOX_INNER_DIM: f32 = 12.0;
        const CHECKBOX_OUTER_DIM: f32 = 18.0;

        ctrl.set_draw_self(false);
        ctrl.draw_rect(
            Rect::new(
                CHECKBOX_LEFT_PADDING,
                0.5 * self.theme.checkbox_height - 0.5 * CHECKBOX_OUTER_DIM,
                CHECKBOX_OUTER_DIM,
                CHECKBOX_OUTER_DIM,
            ),
            Rect::ZERO,
            handle_color,
            texture_id,
        );

        if *self.value {
            ctrl.draw_rect(
                Rect::new(
                    CHECKBOX_LEFT_PADDING + 0.5 * (CHECKBOX_OUTER_DIM - CHECKBOX_INNER_DIM),
                    0.5 * self.theme.checkbox_height - 0.5 * CHECKBOX_INNER_DIM,
                    CHECKBOX_INNER_DIM,
                    CHECKBOX_INNER_DIM,
                ),
                Rect::ZERO,
                0xffffffff,
                texture_id,
            );
        }

        ctrl.draw_text_fitted(
            self.label,
            Align::Start,
            Align::Center,
            Wrap::Word,
            text_color,
            Rect::new(
                40.0,
                0.0,
                f32::max(width - 40.0, 0.0),
                self.theme.checkbox_height,
            ),
        );

        frame.pop_ctrl();

        changed
    }
}
