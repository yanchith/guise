use core::convert::TryInto;
use core::fmt::Debug;

use crate::core::{Ctrl, CtrlFlags, Frame, Layout, Rect};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

pub fn begin_panel<'f, W, H>(
    frame: &'f mut Frame,
    id: u32,
    theme: &Theme,
    layout: Layout,
    width: W,
    height: H,
) -> Option<Ctrl<'f>>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
{
    Panel::new(id, theme, layout, width, height).begin(frame)
}

// TODO(yan): Decide if we want an RAII thing, or an explicit end for widgets
pub fn end_panel(frame: &mut Frame) {
    frame.pop_ctrl();
}

pub struct Panel<'a> {
    id: u32,
    theme: &'a Theme,
    layout: Layout,

    width: Size,
    height: Size,
}

impl<'a> Panel<'a> {
    pub fn new<W, H>(id: u32, theme: &'a Theme, layout: Layout, width: W, height: H) -> Self
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
            theme,
            layout,

            width,
            height,
        }
    }

    pub fn begin<'f>(&self, frame: &'f mut Frame) -> Option<Ctrl<'f>> {
        let parent_extents = frame.ctrl_inner_extents();

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL);
        ctrl.set_layout(self.layout);
        ctrl.set_rect(Rect::new(
            0.0,
            0.0,
            self.width.resolve(parent_extents.x),
            self.height.resolve(parent_extents.y),
        ));
        ctrl.set_padding(self.theme.panel_padding);
        ctrl.set_border(self.theme.panel_border);
        ctrl.set_margin(0.0);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(self.theme.panel_border_color);
        ctrl.set_draw_self_background_color(self.theme.panel_background_color);

        // TODO(yan): Will this ever be None?
        Some(ctrl)
    }

    pub fn end(&self, frame: &mut Frame) {
        frame.pop_ctrl();
    }
}
