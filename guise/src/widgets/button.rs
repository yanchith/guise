use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;
use crate::widgets::tooltip;

pub fn button<A: Allocator + Clone>(frame: &mut Frame<A>, id: u32, label: &str) -> bool {
    do_button(frame, id, label, None, None, &Theme::DEFAULT)
}

pub fn button_with_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    theme: &Theme,
) -> bool {
    do_button(frame, id, label, None, None, theme)
}

pub fn button_with_tooltip<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    tooltip: &str,
) -> bool {
    do_button(frame, id, label, None, Some(tooltip), &Theme::DEFAULT)
}

pub fn button_with_tooltip_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    tooltip: &str,
    theme: &Theme,
) -> bool {
    do_button(frame, id, label, None, Some(tooltip), theme)
}

pub fn image_button<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    image_texture_id: u64,
) -> bool {
    do_button(frame, id, "", Some(image_texture_id), None, &Theme::DEFAULT)
}

pub fn image_button_with_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    image_texture_id: u64,
    theme: &Theme,
) -> bool {
    do_button(frame, id, "", Some(image_texture_id), None, theme)
}

pub fn image_button_with_tooltip<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    image_texture_id: u64,
    tooltip: &str,
) -> bool {
    do_button(
        frame,
        id,
        "",
        Some(image_texture_id),
        Some(tooltip),
        &Theme::DEFAULT,
    )
}

pub fn image_button_with_tooltip_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    image_texture_id: u64,
    tooltip: &str,
    theme: &Theme,
) -> bool {
    do_button(frame, id, "", Some(image_texture_id), Some(tooltip), theme)
}

fn do_button<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    image_texture_id: Option<u64>,
    tooltip: Option<&str>,
    theme: &Theme,
) -> bool {
    let parent_size = frame.ctrl_inner_size();
    let lmb_pressed = frame.inputs_pressed() == Inputs::MB_LEFT;
    let lmb_released = frame.inputs_released() == Inputs::MB_LEFT;

    let (width, height, border, margin) = if image_texture_id.is_some() {
        (
            theme.image_button_width,
            theme.image_button_height,
            theme.image_button_border,
            theme.image_button_margin,
        )
    } else {
        (
            f32::max(0.0, parent_size.x - 2.0 * theme.button_margin),
            theme.button_height,
            theme.button_border,
            theme.button_margin,
        )
    };

    let mut ctrl = frame.push_ctrl(id);
    ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
    ctrl.set_layout(Layout::Vertical);
    ctrl.set_rect(Rect::new(0.0, 0.0, width, height));
    ctrl.set_padding(0.0);
    ctrl.set_border(border);
    ctrl.set_margin(margin);

    let hovered = ctrl.is_hovered();
    let active = ctrl.is_active();

    let (active, changed) = if active && lmb_released {
        ctrl.set_active(false);
        if hovered {
            // Make the control inactive once again after release, as the
            // platform may not be running us on every frame, but only for
            // new events. Also better latency this way.
            (false, true)
        } else {
            (false, false)
        }
    } else if hovered && lmb_pressed {
        ctrl.set_active(true);
        (true, false)
    } else {
        (active, false)
    };

    let (text_color, background_color, border_color) =
        match (image_texture_id.is_some(), hovered, active) {
            (false, false, false) => (
                theme.button_text_color,
                theme.button_background_color,
                theme.button_border_color,
            ),
            (false, true, false) => (
                theme.button_text_color_hovered,
                theme.button_background_color_hovered,
                theme.button_border_color_hovered,
            ),
            (false, _, true) => (
                theme.button_text_color_active,
                theme.button_background_color_active,
                theme.button_border_color_active,
            ),
            (true, false, false) => (
                0,
                theme.image_button_background_color,
                theme.image_button_border_color,
            ),
            (true, true, false) => (
                0,
                theme.image_button_background_color_hovered,
                theme.image_button_border_color_hovered,
            ),
            (true, _, true) => (
                0,
                theme.image_button_background_color_active,
                theme.image_button_border_color_active,
            ),
        };

    ctrl.set_draw_self(true);
    ctrl.set_draw_self_border_color(border_color);
    ctrl.set_draw_self_background_color(background_color);

    if let Some(image_texture_id) = image_texture_id {
        ctrl.draw_rect(
            Rect::new(0.0, 0.0, width, height),
            Rect::ONE,
            0xffffffff,
            image_texture_id,
        )
    } else {
        ctrl.draw_text(label, Align::Center, Align::Center, Wrap::Word, text_color);
    }

    if let Some(tooltip) = tooltip {
        if hovered {
            tooltip::tooltip_with_theme(frame, 0, tooltip, theme);
        }
    }

    frame.pop_ctrl();

    changed
}
