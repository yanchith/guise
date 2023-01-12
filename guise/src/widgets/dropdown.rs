use core::alloc::Allocator;
use core::convert::AsRef;

use crate::convert::cast_u32;
use crate::core::{Align, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::button::button;
use crate::widgets::theme::Theme;

// TODO(yan): Searchable dropdown, analogous to autocomplete text input?

// TODO(yan): Consider a more granular api, where opening the dropdown is
// independent from drawing its contents. Something like:
//
// if let Some(dropdown) = guise::dropdown(frame, line!(), "Label", &state.selected_option.label, &mut state.open) {
//     for option in &options {
//         if guise::button(frame, line!(), option.label) {
//             state.selected_option = option;
//         }
//     }
//
//     dropdown.end();
// }
//

const LABEL_WIDTH_RATIO: f32 = 0.35;
const LABEL_SPACING: f32 = 5.0;

#[inline]
pub fn dropdown<T, A>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    options: &[T],
    selected: &mut Option<usize>,
) -> bool
where
    T: AsRef<str>,
    A: Allocator + Clone,
{
    do_dropdown_and_take_out_trash(frame, id, label, options, selected, false, &Theme::DEFAULT)
}

#[inline]
pub fn dropdown_with_theme<T, A>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    options: &[T],
    selected: &mut Option<usize>,
    theme: &Theme,
) -> bool
where
    T: AsRef<str>,
    A: Allocator + Clone,
{
    do_dropdown_and_take_out_trash(frame, id, label, options, selected, false, theme)
}

#[inline]
pub fn dropdown_with_unselect<T, A>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    options: &[T],
    selected: &mut Option<usize>,
) -> bool
where
    T: AsRef<str>,
    A: Allocator + Clone,
{
    do_dropdown_and_take_out_trash(frame, id, label, options, selected, true, &Theme::DEFAULT)
}

#[inline]
pub fn dropdown_with_unselect_theme<T, A>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    options: &[T],
    selected: &mut Option<usize>,
    theme: &Theme,
) -> bool
where
    T: AsRef<str>,
    A: Allocator + Clone,
{
    do_dropdown_and_take_out_trash(frame, id, label, options, selected, true, theme)
}

fn do_dropdown_and_take_out_trash<T, A>(
    frame: &mut Frame<A>,
    id: u32,
    label: &str,
    options: &[T],
    selected: &mut Option<usize>,
    allow_unselect: bool,
    theme: &Theme,
) -> bool
where
    T: AsRef<str>,
    A: Allocator + Clone,
{
    const OVERLAY_SPACING: f32 = 5.0;

    let parent_size = frame.ctrl_inner_size();
    let window_size = frame.window_size();
    let cursor_position = frame.cursor_position();
    let lmb_pressed = frame.inputs_pressed() == Inputs::MB_LEFT;

    let outer_width = f32::max(0.0, parent_size.x - 2.0 * theme.dropdown_margin);
    let label_width = LABEL_WIDTH_RATIO * outer_width;
    let inner_width = f32::max(0.0, outer_width - label_width - LABEL_SPACING);

    let mut outer_ctrl = frame.push_ctrl(id);
    outer_ctrl.set_flags(CtrlFlags::NONE);
    outer_ctrl.set_layout(Layout::Horizontal);
    outer_ctrl.set_rect(Rect::new(0.0, 0.0, outer_width, theme.dropdown_height));
    outer_ctrl.set_padding(0.0);
    outer_ctrl.set_border(0.0);
    outer_ctrl.set_margin(theme.dropdown_margin);

    outer_ctrl.set_draw_self(false);
    outer_ctrl.draw_text_fitted(
        label,
        Align::Start,
        Align::Center,
        Wrap::Word,
        theme.dropdown_text_color,
        Rect::new(0.0, 0.0, label_width, theme.dropdown_height),
    );

    let mut active_area_ctrl = frame.push_ctrl(0);
    active_area_ctrl.set_flags(CtrlFlags::CAPTURE_HOVER | CtrlFlags::CAPTURE_ACTIVE);
    active_area_ctrl.set_layout(Layout::Vertical);
    active_area_ctrl.set_rect(Rect::new(
        label_width + LABEL_SPACING,
        0.0,
        inner_width,
        theme.dropdown_height,
    ));
    active_area_ctrl.set_padding(0.0);
    active_area_ctrl.set_border(theme.dropdown_border);
    active_area_ctrl.set_margin(0.0);

    let absolute_position = active_area_ctrl.absolute_position();

    let overlay_y = absolute_position.y + theme.dropdown_height + OVERLAY_SPACING;

    let available_height_up = overlay_y;
    let available_height_down = f32::max(window_size.y - overlay_y, 0.0);

    let overlay_height_requested = f32::min(
        options.len() as f32 * (theme.button_height + 2.0 * theme.button_margin),
        theme.dropdown_overlay_max_height,
    );

    let overlay_rect = if overlay_height_requested > available_height_down {
        if available_height_down > available_height_up {
            Rect::new(
                absolute_position.x,
                overlay_y,
                inner_width,
                available_height_down,
            )
        } else {
            let height = f32::min(available_height_up, overlay_height_requested);
            Rect::new(
                absolute_position.x,
                absolute_position.y - height - OVERLAY_SPACING,
                inner_width,
                height,
            )
        }
    } else {
        Rect::new(
            absolute_position.x,
            overlay_y,
            inner_width,
            overlay_height_requested,
        )
    };

    let hovered = active_area_ctrl.is_hovered();
    let mut active = active_area_ctrl.is_active();

    let state = active_area_ctrl.state_mut();
    let mut open = open(state);

    if lmb_pressed {
        if open {
            if !overlay_rect.contains_point(cursor_position) {
                set_open(state, false);
                active_area_ctrl.set_active(false);
                active = false;
                open = false;
            }
        } else if hovered {
            set_open(state, true);
            active_area_ctrl.set_active(true);
            active = true;
            open = true;
        }
    }

    let (text_color, background_color, border_color) = match (hovered, active) {
        (false, false) => (
            theme.dropdown_text_color,
            theme.dropdown_background_color,
            theme.dropdown_border_color,
        ),
        (true, false) => (
            theme.dropdown_text_color_hovered,
            theme.dropdown_background_color_hovered,
            theme.dropdown_border_color_hovered,
        ),
        (_, true) => (
            theme.dropdown_text_color_active,
            theme.dropdown_background_color_active,
            theme.dropdown_border_color_active,
        ),
    };

    active_area_ctrl.set_draw_self(true);
    active_area_ctrl.set_draw_self_border_color(border_color);
    active_area_ctrl.set_draw_self_background_color(background_color);

    let label = if let Some(selected) = selected {
        options[*selected].as_ref()
    } else {
        ""
    };

    active_area_ctrl.draw_text(label, Align::Center, Align::Center, Wrap::Word, text_color);

    let mut changed = false;

    if open {
        frame.begin_overlay();

        let mut ctrl = frame.push_ctrl(id);
        ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(overlay_rect);

        // Margin is zero, because we are setting an absolute position.
        ctrl.set_padding(0.0);
        ctrl.set_border(theme.dropdown_border);
        ctrl.set_margin(0.0);

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(theme.dropdown_border_color_active);
        ctrl.set_draw_self_background_color(theme.dropdown_background_color_active);

        if allow_unselect {
            if button(frame, 0, "") {
                *selected = None;
                changed = true;
            }
        }

        for (i, option) in options.iter().enumerate() {
            if button(frame, 1 + cast_u32(i), option.as_ref()) {
                *selected = Some(i);
                changed = true;
            }
        }

        frame.pop_ctrl();

        frame.end_overlay();
    }

    if changed {
        set_open(frame.ctrl_state_mut(), false);
    }

    frame.pop_ctrl();
    frame.pop_ctrl();

    changed
}

fn open(state: &CtrlState) -> bool {
    state[0] == 1
}

fn set_open(state: &mut CtrlState, open: bool) {
    state[0] = u8::from(open)
}
