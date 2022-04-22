use core::alloc::Allocator;
use core::fmt::Debug;

use crate::core::{Ctrl, CtrlFlags, Frame, Layout, Rect};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

pub fn begin_panel<'f, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    height: H,
) -> Ctrl<'f, A>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    Panel::new(id, width, height).begin(frame)
}

// TODO(yan): Decide if we want an RAII thing, or an explicit end for widgets
pub fn end_panel<A: Allocator + Clone>(frame: &mut Frame<A>) {
    frame.pop_ctrl();
}

pub struct Panel<'a> {
    id: u32,
    width: Size,
    height: Size,

    layout: Layout,

    theme: &'a Theme,
}

impl<'a> Panel<'a> {
    pub fn new<W, H>(id: u32, width: W, height: H) -> Self
    where
        W: TryInto<Size>,
        H: TryInto<Size>,
        <W as TryInto<Size>>::Error: Debug,
        <H as TryInto<Size>>::Error: Debug,
    {
        let width = width.try_into().unwrap();
        let height = height.try_into().unwrap();

        Self {
            id,
            width,
            height,

            layout: Layout::Vertical,

            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_layout(&mut self, layout: Layout) -> &mut Self {
        self.layout = layout;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn begin<'f, A: Allocator + Clone>(&self, frame: &'f mut Frame<A>) -> Ctrl<'f, A> {
        let parent_size = frame.ctrl_inner_size();

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL);
        ctrl.set_layout(self.layout);
        ctrl.set_rect(Rect::new(
            0.0,
            0.0,
            self.width.resolve(parent_size.x),
            self.height.resolve(parent_size.y),
        ));
        ctrl.set_padding(self.theme.panel_padding);
        ctrl.set_border(self.theme.panel_border);
        ctrl.set_margin(0.0);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(self.theme.panel_border_color);
        ctrl.set_draw_self_background_color(self.theme.panel_background_color);

        ctrl
    }

    pub fn end<A: Allocator + Clone>(&self, frame: &mut Frame<A>) {
        frame.pop_ctrl();
    }
}
