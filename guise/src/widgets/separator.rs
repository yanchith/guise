use core::alloc::Allocator;

use crate::core::{CtrlFlags, Frame, Layout, Rect};
use crate::widgets::theme::Theme;

pub fn separator<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32) {
    Separator::new(id).show(frame);
}

pub struct Separator<'a> {
    id: u32,
    theme: &'a Theme,
}

impl<'a> Separator<'a> {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&self, frame: &mut Frame<A>) {
        let parent_size = frame.ctrl_inner_size();

        let x = parent_size.x * 0.1 - self.theme.separator_margin;
        let width = f32::max(0.0, parent_size.x * 0.8 - self.theme.separator_margin);

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::NONE);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(x, 0.0, width, self.theme.separator_height));
        ctrl.set_padding(0.0);
        ctrl.set_border(0.0);
        ctrl.set_margin(self.theme.separator_margin);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_background_color(self.theme.separator_color);

        frame.pop_ctrl();
    }
}
