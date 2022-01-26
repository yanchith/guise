use core::alloc::Allocator;
use core::fmt::Debug;

use crate::core::{Ctrl, CtrlFlags, Frame, Layout, Rect};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

pub fn begin_panel<'f, W, H, A, TA>(
    frame: &'f mut Frame<A, TA>,
    id: u32,
    width: W,
    height: H,
) -> Ctrl<'f, A, TA>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
    TA: Allocator,
{
    Panel::new(id, Layout::Vertical, width, height).begin(frame)
}

pub fn begin_panel_ex<'f, W, H, A, TA>(
    frame: &'f mut Frame<A, TA>,
    id: u32,
    width: W,
    height: H,
    layout: Layout,
) -> Ctrl<'f, A, TA>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
    TA: Allocator,
{
    Panel::new(id, layout, width, height).begin(frame)
}

// TODO(yan): Decide if we want an RAII thing, or an explicit end for widgets
pub fn end_panel<A, TA>(frame: &mut Frame<A, TA>)
where
    A: Allocator + Clone,
    TA: Allocator,
{
    frame.pop_ctrl();
}

pub struct Panel<'a> {
    id: u32,
    layout: Layout,

    theme: &'a Theme,
    width: Size,
    height: Size,
}

impl<'a> Panel<'a> {
    pub fn new<W, H>(id: u32, layout: Layout, width: W, height: H) -> Self
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
            layout,

            theme: &Theme::DEFAULT,
            width,
            height,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn begin<'f, A, TA>(&self, frame: &'f mut Frame<A, TA>) -> Ctrl<'f, A, TA>
    where
        A: Allocator + Clone,
        TA: Allocator,
    {
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

    pub fn end<A, TA>(&self, frame: &mut Frame<A, TA>)
    where
        A: Allocator + Clone,
        TA: Allocator,
    {
        frame.pop_ctrl();
    }
}
