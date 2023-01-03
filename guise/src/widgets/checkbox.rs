use core::alloc::Allocator;

use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

#[inline]
pub fn checkbox<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut bool,
    label: &str,
) -> bool {
    checkbox_with_theme(frame, id, value, label, &Theme::DEFAULT)
}

pub fn checkbox_with_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut bool,
    label: &str,
    theme: &Theme,
) -> bool {
    let texture_id = frame.font_atlas_texture_id();
    let parent_size = frame.ctrl_inner_size();
    let lmb_pressed = frame.inputs_pressed() == Inputs::MB_LEFT;
    let lmb_released = frame.inputs_released() == Inputs::MB_LEFT;

    let width = f32::max(0.0, parent_size.x - 2.0 * theme.checkbox_margin);

    let mut ctrl = frame.push_ctrl(id);
    ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
    ctrl.set_layout(Layout::Vertical);
    ctrl.set_rect(Rect::new(0.0, 0.0, width, theme.checkbox_height));
    ctrl.set_padding(0.0);
    ctrl.set_border(theme.checkbox_border);
    ctrl.set_margin(theme.checkbox_margin);

    let hovered = ctrl.is_hovered();
    let active = ctrl.is_active();

    let (active, changed) = if active && lmb_released {
        ctrl.set_active(false);
        if hovered {
            // Make the control inactive once again after release, as the
            // platform may not be running us on every frame, but only for
            // new events. Also better latency this way.
            *value = !*value;
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

    let (handle_color, text_color) = match (hovered, active) {
        (false, false) => (theme.checkbox_handle_color, theme.checkbox_text_color),
        (true, false) => (
            theme.checkbox_handle_color_hovered,
            theme.checkbox_text_color_hovered,
        ),
        (_, true) => (
            theme.checkbox_handle_color_active,
            theme.checkbox_text_color_active,
        ),
    };

    const CHECKBOX_LEFT_PADDING: f32 = 5.0;
    const CHECKBOX_INNER_DIM: f32 = 12.0;
    const CHECKBOX_OUTER_DIM: f32 = 18.0;

    ctrl.set_draw_self(false);
    ctrl.draw_rect(
        Rect::new(
            CHECKBOX_LEFT_PADDING,
            0.5 * theme.checkbox_height - 0.5 * CHECKBOX_OUTER_DIM,
            CHECKBOX_OUTER_DIM,
            CHECKBOX_OUTER_DIM,
        ),
        Rect::ZERO,
        handle_color,
        texture_id,
    );

    if *value {
        ctrl.draw_rect(
            Rect::new(
                CHECKBOX_LEFT_PADDING + 0.5 * (CHECKBOX_OUTER_DIM - CHECKBOX_INNER_DIM),
                0.5 * theme.checkbox_height - 0.5 * CHECKBOX_INNER_DIM,
                CHECKBOX_INNER_DIM,
                CHECKBOX_INNER_DIM,
            ),
            Rect::ZERO,
            0xffffffff,
            texture_id,
        );
    }

    ctrl.draw_text_fitted(
        label,
        Align::Start,
        Align::Center,
        Wrap::Word,
        text_color,
        Rect::new(
            40.0,
            0.0,
            f32::max(width - 40.0, 0.0),
            theme.checkbox_height,
        ),
    );

    frame.pop_ctrl();

    changed
}
