use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Vec2, Wrap};
use crate::widgets::theme::Theme;

pub fn button<A, TA>(frame: &mut Frame<A, TA>, id: u32, label: &str) -> bool
where
    A: Allocator + Clone,
    TA: Allocator,
{
    Button::new(id, label).show(frame)
}

// TODO(yan): Split ImageButton from Button, because their themes are already
// separate.

pub struct Button<'a> {
    id: u32,
    label: &'a str,
    image_texture_id: Option<u64>,
    theme: &'a Theme,
}

impl<'a> Button<'a> {
    pub fn new(id: u32, label: &'a str) -> Self {
        Self {
            id,
            label,
            image_texture_id: None,
            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn set_image(&mut self, image_texture_id: u64) -> &mut Self {
        self.image_texture_id = Some(image_texture_id);
        self
    }

    pub fn show<A, TA>(&self, frame: &mut Frame<A, TA>) -> bool
    where
        A: Allocator + Clone,
        TA: Allocator,
    {
        let parent_size = frame.ctrl_inner_size();
        let lmb_pressed = frame.inputs_pressed() == Inputs::MOUSE_BUTTON_LEFT;
        let lmb_released = frame.inputs_released() == Inputs::MOUSE_BUTTON_LEFT;

        let (width, height, border, margin) = if self.image_texture_id.is_some() {
            (
                self.theme.image_button_width,
                self.theme.image_button_height,
                self.theme.image_button_border,
                self.theme.image_button_margin,
            )
        } else {
            (
                f32::max(0.0, parent_size.x - 2.0 * self.theme.input_text_margin),
                self.theme.button_height,
                self.theme.button_border,
                self.theme.button_margin,
            )
        };

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, width, height));
        ctrl.set_padding(0.0);
        ctrl.set_border(border);
        ctrl.set_margin(margin);

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

        let (text_color, background_color, border_color) =
            match (self.image_texture_id.is_some(), hovered, active) {
                (false, false, false) => (
                    self.theme.button_text_color,
                    self.theme.button_background_color,
                    self.theme.button_border_color,
                ),
                (false, true, false) => (
                    self.theme.button_text_color_hovered,
                    self.theme.button_background_color_hovered,
                    self.theme.button_border_color_hovered,
                ),
                (false, _, true) => (
                    self.theme.button_text_color_active,
                    self.theme.button_background_color_active,
                    self.theme.button_border_color_active,
                ),
                (true, false, false) => (
                    0,
                    self.theme.image_button_background_color,
                    self.theme.image_button_border_color,
                ),
                (true, true, false) => (
                    0,
                    self.theme.image_button_background_color_hovered,
                    self.theme.image_button_border_color_hovered,
                ),
                (true, _, true) => (
                    0,
                    self.theme.image_button_background_color_active,
                    self.theme.image_button_border_color_active,
                ),
            };

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(border_color);
        ctrl.set_draw_self_background_color(background_color);

        if let Some(image_texture_id) = self.image_texture_id {
            ctrl.draw_rect(
                false,
                Rect::new(-border, -border, width, height),
                Rect::ONE,
                0xffffffff,
                image_texture_id,
            )
        } else {
            ctrl.draw_text_ex(
                false,
                Vec2::ZERO,
                self.label,
                Align::Center,
                Align::Center,
                Wrap::Word,
                text_color,
            );
        }

        frame.pop_ctrl();

        changed
    }
}
