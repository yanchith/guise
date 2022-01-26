use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Layout, Rect, Vec2, Wrap};
use crate::widgets::theme::Theme;

const FLAGS: CtrlFlags =
    CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER | CtrlFlags::SHRINK_TO_FIT_INLINE_CONTENT;

pub fn text<A, TA>(frame: &mut Frame<A, TA>, id: u32, text: &str)
where
    A: Allocator + Clone,
    TA: Allocator,
{
    Text::new(id, text).show(frame);
}

pub fn text_ex<A, TA>(frame: &mut Frame<A, TA>, id: u32, text: &str, horizontal_align: Align)
where
    A: Allocator + Clone,
    TA: Allocator,
{
    Text::new(id, text)
        .set_horizontal_align(horizontal_align)
        .show(frame);
}

pub struct Text<'a> {
    id: u32,
    text: &'a str,
    theme: &'a Theme,
    horizontal_align: Align,
}

impl<'a> Text<'a> {
    pub fn new(id: u32, text: &'a str) -> Self {
        Self {
            id,
            text,
            theme: &Theme::DEFAULT,
            horizontal_align: Align::Start,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn set_horizontal_align(&mut self, horizontal_align: Align) -> &mut Self {
        self.horizontal_align = horizontal_align;
        self
    }

    pub fn show<A, TA>(&self, frame: &mut Frame<A, TA>)
    where
        A: Allocator + Clone,
        TA: Allocator,
    {
        let parent_size = frame.ctrl_inner_size();

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(FLAGS);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, parent_size.x, parent_size.y));
        ctrl.set_padding(self.theme.text_padding);
        ctrl.set_border(self.theme.text_border);
        ctrl.set_margin(self.theme.text_margin);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(self.theme.text_border_color);
        ctrl.set_draw_self_background_color(self.theme.text_background_color);
        ctrl.draw_text_ex(
            true,
            Vec2::ZERO,
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
