use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

// TODO(yan): This is a text-only tooltip for now, because we at the moment we
// can only shrink the component to fit inline content. We could do shrinking
// (or even resizing, so we don't have to care about initial values) with any
// content by using the results of the layout pass for rendering.
pub struct TooltipText<'a> {
    id: u32,
    text: &'a str,
    horizontal_align: Align,
    theme: &'a Theme,
}

impl<'a> TooltipText<'a> {
    pub fn new(id: u32, text: &'a str) -> Self {
        Self {
            id,
            text,
            horizontal_align: Align::Start,
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
        frame.begin_overlay();

        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();

        let border = self.theme.text_tooltip_border;
        let padding = self.theme.text_tooltip_padding;

        let mut ctrl = frame.push_ctrl(self.id);
        // NB: Text doesn't capture scrolling, because it actually slightly
        // overflows - by the value of its border (and padding, if we had set
        // it), but because Ctrl::draw_text does its own aligning and insetting,
        // this is never visible.
        ctrl.set_flags(CtrlFlags::ALL_RESIZE_TO_FIT);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(
            cursor_position.x,
            cursor_position.y,
            // NB: Set to parent size so that the text layout can happen with
            // realistic clipping. This rect is however resized to fit the text
            // during the control layout phase.
            f32::max(0.0, parent_size.x - cursor_position.x),
            parent_size.y,
        ));
        // NB: Padding is not set, because there's no child controls, and the
        // text layout computes uses its own inset.
        ctrl.set_border(border);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(self.theme.text_tooltip_border_color);
        ctrl.set_draw_self_background_color(self.theme.text_tooltip_background_color);
        ctrl.draw_text_inset_and_extend_content_rect(
            self.text,
            self.horizontal_align,
            // Vertical align does not make sense with shrunk-to-fit controls.
            Align::Start,
            Wrap::Word,
            self.theme.text_tooltip_text_color,
            border + padding,
        );

        frame.pop_ctrl();

        frame.end_overlay();
    }
}
