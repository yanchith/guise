use core::alloc::Allocator;

use crate::core::{CtrlFlags, Frame, Layout, Rect};
use crate::widgets::theme::Theme;

#[inline]
pub fn separator<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32) {
    separator_with_theme(frame, id, &Theme::DEFAULT)
}

pub fn separator_with_theme<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32, theme: &Theme) {
    let parent_size = frame.ctrl_inner_size();

    let x = parent_size.x * 0.1 - theme.separator_margin;
    let width = f32::max(0.0, parent_size.x * 0.8 - theme.separator_margin);

    let mut ctrl = frame.push_ctrl(id);
    ctrl.set_flags(CtrlFlags::NONE);
    ctrl.set_layout(Layout::Vertical);
    ctrl.set_rect(Rect::new(x, 0.0, width, theme.separator_height));
    ctrl.set_padding(0.0);
    ctrl.set_border(0.0);
    ctrl.set_margin(theme.separator_margin);

    ctrl.set_draw_self(true);
    ctrl.set_draw_self_background_color(theme.separator_color);

    frame.pop_ctrl();
}
