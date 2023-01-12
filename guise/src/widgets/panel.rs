use core::alloc::Allocator;
use core::fmt::Debug;

use crate::core::{Align, Ctrl, CtrlFlags, Frame, Layout, Rect, Wrap};
use crate::widgets::size::Size;
use crate::widgets::theme::Theme;

const DEFAULT_OPTIONS: PanelOptions = PanelOptions {
    draw_padding: true,
    draw_border: true,
    draw_header: true,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelOptions {
    pub draw_padding: bool,
    pub draw_border: bool,
    pub draw_header: bool,
}

impl Default for PanelOptions {
    fn default() -> Self {
        DEFAULT_OPTIONS
    }
}

#[inline]
pub fn begin_panel<'f, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    height: H,
    label: &str,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame,
        id,
        width,
        height,
        label,
        Layout::Vertical,
        false,
        &DEFAULT_OPTIONS,
        &Theme::DEFAULT,
    );

    Some((Panel(false), ctrl))
}

#[inline]
pub fn begin_panel_with_layout<'f, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    height: H,
    label: &str,
    layout: Layout,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame,
        id,
        width,
        height,
        label,
        layout,
        false,
        &DEFAULT_OPTIONS,
        &Theme::DEFAULT,
    );

    Some((Panel(false), ctrl))
}

#[inline]
pub fn begin_panel_with_fit_height<'f, W, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    label: &str,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame,
        id,
        width,
        Size::new_absolute(0.0),
        label,
        Layout::Vertical,
        true,
        &DEFAULT_OPTIONS,
        &Theme::DEFAULT,
    );

    Some((Panel(false), ctrl))
}

#[inline]
pub fn begin_panel_with_layout_fit_height<'f, W, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    label: &str,
    layout: Layout,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame,
        id,
        width,
        Size::new_absolute(0.0),
        label,
        layout,
        true,
        &DEFAULT_OPTIONS,
        &Theme::DEFAULT,
    );

    Some((Panel(false), ctrl))
}

#[inline]
pub fn begin_panel_with_layout_options<'f, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    height: H,
    label: &str,
    layout: Layout,
    options: &PanelOptions,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame,
        id,
        width,
        height,
        label,
        layout,
        false,
        options,
        &Theme::DEFAULT,
    );

    Some((Panel(false), ctrl))
}

#[inline]
pub fn begin_panel_with_layout_fit_height_options<'f, W, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    label: &str,
    layout: Layout,
    options: &PanelOptions,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame,
        id,
        width,
        Size::new_absolute(0.0),
        label,
        layout,
        true,
        options,
        &Theme::DEFAULT,
    );

    Some((Panel(false), ctrl))
}

#[inline]
pub fn begin_panel_with_layout_options_theme<'f, W, H, A>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: W,
    height: H,
    label: &str,
    layout: Layout,
    options: &PanelOptions,
    theme: &Theme,
) -> Option<(Panel, Ctrl<'f, A>)>
where
    W: TryInto<Size>,
    H: TryInto<Size>,
    <W as TryInto<Size>>::Error: Debug,
    <H as TryInto<Size>>::Error: Debug,
    A: Allocator + Clone,
{
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();

    let ctrl = do_panel_and_plot_mandelbrot_set(
        frame, id, width, height, label, layout, false, options, theme,
    );

    Some((Panel(false), ctrl))
}

pub struct Panel(bool);

impl Panel {
    pub fn end<A: Allocator + Clone>(mut self, frame: &mut Frame<A>) {
        assert!(!self.0);

        frame.pop_ctrl();
        frame.pop_ctrl();
        self.0 = true;
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        debug_assert!(self.0)
    }
}

fn do_panel_and_plot_mandelbrot_set<'f, A: Allocator + Clone>(
    frame: &'f mut Frame<A>,
    id: u32,
    width: Size,
    height: Size,
    label: &str,
    layout: Layout,
    fit_height: bool,
    options: &PanelOptions,
    theme: &Theme,
) -> Ctrl<'f, A> {
    let parent_size = frame.ctrl_inner_size();
    let outer_flags = if fit_height {
        CtrlFlags::RESIZE_TO_FIT_VERTICAL
    } else {
        CtrlFlags::NONE
    };
    let body_flags = if fit_height {
        CtrlFlags::CAPTURE_SCROLL | CtrlFlags::RESIZE_TO_FIT_VERTICAL
    } else {
        CtrlFlags::CAPTURE_SCROLL
    };

    let outer_width = f32::max(0.0, width.resolve(parent_size.x) - 2.0 * theme.panel_margin);
    let outer_height = f32::max(
        0.0,
        height.resolve(parent_size.y) - 2.0 * theme.panel_margin,
    );

    let mut outer_ctrl = frame.push_ctrl(id);
    outer_ctrl.set_flags(outer_flags);
    outer_ctrl.set_layout(Layout::Vertical);
    outer_ctrl.set_rect(Rect::new(0.0, 0.0, outer_width, outer_height));

    outer_ctrl.set_padding(0.0);
    outer_ctrl.set_border(if options.draw_border {
        theme.panel_border
    } else {
        0.0
    });
    outer_ctrl.set_margin(theme.panel_margin);

    if options.draw_border {
        outer_ctrl.set_draw_self(true);
        outer_ctrl.set_draw_self_border_color(theme.panel_border_color);
    }

    if options.draw_header {
        let mut header_ctrl = frame.push_ctrl(0);
        header_ctrl.set_flags(CtrlFlags::NONE);
        header_ctrl.set_layout(Layout::Free);
        header_ctrl.set_rect(Rect::new(0.0, 0.0, outer_width, theme.panel_header_height));
        header_ctrl.set_padding(0.0);
        header_ctrl.set_border(0.0);
        header_ctrl.set_margin(0.0);

        header_ctrl.set_draw_self(true);
        header_ctrl.set_draw_self_background_color(theme.panel_header_background_color);

        if label.len() > 0 {
            header_ctrl.draw_text(
                label,
                Align::Center,
                Align::Center,
                Wrap::Word,
                theme.panel_header_text_color,
            );
        }

        frame.pop_ctrl();
    }

    let mut body_ctrl = frame.push_ctrl(1);
    body_ctrl.set_flags(body_flags);
    body_ctrl.set_layout(layout);
    body_ctrl.set_rect(Rect::new(
        0.0,
        0.0,
        outer_width,
        if options.draw_header {
            f32::max(0.0, outer_height - theme.panel_header_height)
        } else {
            outer_height
        },
    ));
    body_ctrl.set_padding(if options.draw_padding {
        theme.panel_padding
    } else {
        0.0
    });
    body_ctrl.set_border(0.0);
    body_ctrl.set_margin(0.0);

    body_ctrl.set_draw_self(true);
    body_ctrl.set_draw_self_border_color(theme.panel_border_color);
    body_ctrl.set_draw_self_background_color(theme.panel_background_color);

    body_ctrl
}
