use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

pub fn tooltip<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32, text: &str) {
    tooltip_with_theme(frame, id, text, &Theme::DEFAULT)
}

pub fn tooltip_with_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    text: &str,
    theme: &Theme,
) {
    frame.begin_overlay();

    let parent_size = frame.ctrl_inner_size();
    let cursor_position = frame.cursor_position();

    let mut ctrl = frame.push_ctrl(id);
    ctrl.set_flags(CtrlFlags::ALL_RESIZE_TO_FIT);
    ctrl.set_layout(Layout::Vertical);
    ctrl.set_rect(Rect::new(
        cursor_position.x,
        cursor_position.y,
        // Set to parent size so that the text layout can happen with realistic
        // clipping. This rect is however resized to fit the text during the
        // layout phase.
        f32::max(0.0, parent_size.x - cursor_position.x),
        parent_size.y,
    ));
    // Padding is not set, because there's no child controls, and the text
    // layout computes uses its own inset.
    ctrl.set_border(theme.text_tooltip_border);

    ctrl.set_draw_self(true);
    ctrl.set_draw_self_border_color(theme.text_tooltip_border_color);
    ctrl.set_draw_self_background_color(theme.text_tooltip_background_color);
    ctrl.draw_text_inset_and_extend_content_rect(
        text,
        // Horizontal aligns don't make much sense with text tooltips.
        Align::Start,
        // Vertical align does not make sense with shrunk-to-fit controls.
        Align::Start,
        Wrap::Word,
        theme.text_tooltip_text_color,
        theme.text_tooltip_border + theme.text_tooltip_padding,
    );

    frame.pop_ctrl();

    frame.end_overlay();
}
