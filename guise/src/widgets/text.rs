use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

pub fn text<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32, text: &str) {
    Text::new(id, text).show(frame);
}

pub struct Text<'a> {
    id: u32,
    text: &'a str,
    horizontal_align: Align,
    theme: &'a Theme,
}

impl<'a> Text<'a> {
    pub fn new(id: u32, text: &'a str) -> Self {
        Self {
            id,
            text,
            horizontal_align: Align::Center,
            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_horizontal_align(&mut self, horizontal_align: Align) -> &mut Self {
        self.horizontal_align = horizontal_align;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&self, frame: &mut Frame<A>) {
        let parent_size = frame.ctrl_inner_size();

        let margin = self.theme.text_margin;
        let border = self.theme.text_border;
        let padding = self.theme.text_padding;

        let mut ctrl = frame.push_ctrl(self.id);
        // NB: Text doesn't capture scrolling, because it actually slightly
        // overflows - by the value of its border (and padding, if we had set
        // it), but because Ctrl::draw_text does its own aligning and insetting,
        // this is never visible.

        ctrl.set_flags(CtrlFlags::ALL_RESIZE_TO_FIT);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, parent_size.x, parent_size.y));

        // Padding is not set through the control, but applied with drawing,
        // because the text layout uses its own inset.
        ctrl.set_border(border);
        ctrl.set_margin(margin);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(self.theme.text_border_color);
        ctrl.set_draw_self_background_color(self.theme.text_background_color);
        ctrl.draw_text(
            true,
            None,
            border + padding,
            self.text,
            self.horizontal_align,
            // Vertical align does not make sense with shrunk-to-fit controls.
            Align::Start,
            Wrap::Word,
            self.theme.text_text_color,
        );

        frame.pop_ctrl();
    }
}
