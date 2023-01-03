use core::alloc::Allocator;
use core::fmt::Debug;

use crate::core::{Align, Ctrl, CtrlFlags, Frame, Layout, Rect, Wrap};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

pub fn begin_panel<'f, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    height: H,
    label: &str,
) -> Ctrl<'f, A>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    Panel::new(id, width, height, label).begin(frame)
}

pub fn end_panel<A: Allocator + Clone>(frame: &mut Frame<A>) {
    frame.pop_ctrl();
    frame.pop_ctrl();
}

pub struct Panel<'a> {
    id: u32,
    width: Size,
    height: Size,
    label: &'a str,

    resize_height_to_fit_content: bool,
    layout: Layout,
    draw_padding: bool,
    draw_border: bool,
    draw_header: bool,

    theme: &'a Theme,
}

impl<'a> Panel<'a> {
    pub fn new<W, H>(id: u32, width: W, height: H, label: &'a str) -> Self
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
            label,

            resize_height_to_fit_content: false,
            layout: Layout::Vertical,
            draw_padding: true,
            draw_border: true,
            draw_header: true,

            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_resize_height_to_fit_content(&mut self, resize: bool) -> &mut Self {
        self.resize_height_to_fit_content = resize;
        self
    }

    pub fn set_layout(&mut self, layout: Layout) -> &mut Self {
        self.layout = layout;
        self
    }

    // This is a shorthand for setting the padding-width to zero in theme.
    pub fn set_draw_padding(&mut self, draw_padding: bool) -> &mut Self {
        self.draw_padding = draw_padding;
        self
    }

    // This is a shorthand for setting the border-width to zero in theme.
    pub fn set_draw_border(&mut self, draw_border: bool) -> &mut Self {
        self.draw_border = draw_border;
        self
    }

    pub fn set_draw_header(&mut self, draw_header: bool) -> &mut Self {
        self.draw_header = draw_header;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn begin<'f, A: Allocator + Clone>(&self, frame: &'f mut Frame<A>) -> Ctrl<'f, A> {
        let parent_size = frame.ctrl_inner_size();
        let outer_flags = if self.resize_height_to_fit_content {
            CtrlFlags::RESIZE_TO_FIT_VERTICAL
        } else {
            CtrlFlags::NONE
        };
        let body_flags = if self.resize_height_to_fit_content {
            CtrlFlags::CAPTURE_SCROLL | CtrlFlags::RESIZE_TO_FIT_VERTICAL
        } else {
            CtrlFlags::CAPTURE_SCROLL
        };

        let outer_width = f32::max(
            0.0,
            self.width.resolve(parent_size.x) - 2.0 * self.theme.panel_margin,
        );
        let outer_height = f32::max(
            0.0,
            self.height.resolve(parent_size.y) - 2.0 * self.theme.panel_margin,
        );

        let mut outer_ctrl = frame.push_ctrl(self.id);
        outer_ctrl.set_flags(outer_flags);
        outer_ctrl.set_layout(Layout::Vertical);
        outer_ctrl.set_rect(Rect::new(0.0, 0.0, outer_width, outer_height));

        outer_ctrl.set_padding(0.0);
        outer_ctrl.set_border(if self.draw_border {
            self.theme.panel_border
        } else {
            0.0
        });
        outer_ctrl.set_margin(self.theme.panel_margin);

        if self.draw_border {
            outer_ctrl.set_draw_self(true);
            outer_ctrl.set_draw_self_border_color(self.theme.panel_border_color);
        }

        if self.draw_header {
            let mut header_ctrl = frame.push_ctrl(0);
            header_ctrl.set_flags(CtrlFlags::NONE);
            header_ctrl.set_layout(Layout::Free);
            header_ctrl.set_rect(Rect::new(
                0.0,
                0.0,
                outer_width,
                self.theme.panel_header_height,
            ));
            header_ctrl.set_padding(0.0);
            header_ctrl.set_border(0.0);
            header_ctrl.set_margin(0.0);

            header_ctrl.set_draw_self(true);
            header_ctrl.set_draw_self_background_color(self.theme.panel_header_background_color);

            if self.label.len() > 0 {
                header_ctrl.draw_text(
                    self.label,
                    Align::Center,
                    Align::Center,
                    Wrap::Word,
                    self.theme.panel_header_text_color,
                );
            }

            frame.pop_ctrl();
        }

        let mut body_ctrl = frame.push_ctrl(1);
        body_ctrl.set_flags(body_flags);
        body_ctrl.set_layout(self.layout);
        body_ctrl.set_rect(Rect::new(
            0.0,
            0.0,
            outer_width,
            if self.draw_header {
                f32::max(0.0, outer_height - self.theme.panel_header_height)
            } else {
                outer_height
            },
        ));
        body_ctrl.set_padding(if self.draw_padding {
            self.theme.panel_padding
        } else {
            0.0
        });
        body_ctrl.set_border(0.0);
        body_ctrl.set_margin(0.0);

        body_ctrl.set_draw_self(true);
        body_ctrl.set_draw_self_border_color(self.theme.panel_border_color);
        body_ctrl.set_draw_self_background_color(self.theme.panel_background_color);

        body_ctrl
    }

    pub fn end<A: Allocator + Clone>(&self, frame: &mut Frame<A>) {
        frame.pop_ctrl();
        frame.pop_ctrl();
    }
}
