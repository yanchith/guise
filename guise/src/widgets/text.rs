use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

#[inline]
pub fn text<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32, text: &str) {
    text_with_align_theme(frame, id, text, Align::Center, &Theme::DEFAULT)
}

#[inline]
pub fn text_with_align<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    text: &str,
    align: Align,
) {
    text_with_align_theme(frame, id, text, align, &Theme::DEFAULT)
}

pub fn text_with_align_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    text: &str,
    align: Align,
    theme: &Theme,
) {
    let parent_size = frame.ctrl_inner_size();

    let mut ctrl = frame.push_ctrl(id);

    ctrl.set_flags(CtrlFlags::ALL_RESIZE_TO_FIT);
    ctrl.set_layout(Layout::Vertical);
    ctrl.set_rect(Rect::new(0.0, 0.0, parent_size.x, parent_size.y));

    // Padding is not set through the control, but applied with drawing,
    // because the text layout uses its own inset.
    ctrl.set_border(theme.text_border);
    ctrl.set_margin(theme.text_margin);

    ctrl.set_draw_self(true);
    ctrl.set_draw_self_border_color(theme.text_border_color);
    ctrl.set_draw_self_background_color(theme.text_background_color);
    ctrl.draw_text_inset_and_extend_content_rect(
        text,
        align,
        // Vertical align does not make sense with shrunk-to-fit controls.
        Align::Start,
        Wrap::Word,
        theme.text_text_color,
        theme.text_border + theme.text_padding,
    );

    frame.pop_ctrl();
}
