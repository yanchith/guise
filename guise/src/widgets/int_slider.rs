use core::alloc::Allocator;
use core::fmt::Write;
use core::mem;
use core::slice;

use arrayvec::ArrayString;

use crate::convert::cast_u32;
use crate::core::{Align, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

pub fn int_slider<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut i32,
    label: &str,
) -> bool {
    do_int_slider_and_take_kids_to_school(
        frame,
        id,
        slice::from_mut(value),
        label,
        1.0,
        i32::MIN,
        i32::MAX,
        &Theme::DEFAULT,
    )
}

pub fn int_slider_with_speed_min_max<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut i32,
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
) -> bool {
    do_int_slider_and_take_kids_to_school(
        frame,
        id,
        slice::from_mut(value),
        label,
        speed,
        min,
        max,
        &Theme::DEFAULT,
    )
}

pub fn int_slider_with_speed_min_max_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut i32,
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
    theme: &Theme,
) -> bool {
    do_int_slider_and_take_kids_to_school(
        frame,
        id,
        slice::from_mut(value),
        label,
        speed,
        min,
        max,
        theme,
    )
}

pub fn int2_slider<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 2],
    label: &str,
) -> bool {
    do_int_slider_and_take_kids_to_school(
        frame,
        id,
        value,
        label,
        1.0,
        i32::MIN,
        i32::MAX,
        &Theme::DEFAULT,
    )
}

pub fn int2_slider_with_speed_min_max<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 2],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
) -> bool {
    do_int_slider_and_take_kids_to_school(frame, id, value, label, speed, min, max, &Theme::DEFAULT)
}

pub fn int2_slider_with_speed_min_max_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 2],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
    theme: &Theme,
) -> bool {
    do_int_slider_and_take_kids_to_school(frame, id, value, label, speed, min, max, theme)
}

pub fn int3_slider<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 3],
    label: &str,
) -> bool {
    do_int_slider_and_take_kids_to_school(
        frame,
        id,
        value,
        label,
        1.0,
        i32::MIN,
        i32::MAX,
        &Theme::DEFAULT,
    )
}

pub fn int3_slider_with_speed_min_max<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 3],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
) -> bool {
    do_int_slider_and_take_kids_to_school(frame, id, value, label, speed, min, max, &Theme::DEFAULT)
}

pub fn int3_slider_with_speed_min_max_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 3],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
    theme: &Theme,
) -> bool {
    do_int_slider_and_take_kids_to_school(frame, id, value, label, speed, min, max, theme)
}

pub fn int4_slider<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 4],
    label: &str,
) -> bool {
    do_int_slider_and_take_kids_to_school(
        frame,
        id,
        value,
        label,
        1.0,
        i32::MIN,
        i32::MAX,
        &Theme::DEFAULT,
    )
}

pub fn int4_slider_with_speed_min_max<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 4],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
) -> bool {
    do_int_slider_and_take_kids_to_school(frame, id, value, label, speed, min, max, &Theme::DEFAULT)
}

pub fn int4_slider_with_speed_min_max_theme<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [i32; 4],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
    theme: &Theme,
) -> bool {
    do_int_slider_and_take_kids_to_school(frame, id, value, label, speed, min, max, theme)
}

fn do_int_slider_and_take_kids_to_school<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value_mut: &mut [i32],
    label: &str,
    speed: f32,
    min: i32,
    max: i32,
    theme: &Theme,
) -> bool {
    const LABEL_WIDTH_RATIO: f32 = 0.35;
    const LABEL_SPACING: f32 = 5.0;
    const INPUT_SPACING: f32 = 2.0;

    let mut s: ArrayString<256> = ArrayString::new();

    let parent_size = frame.ctrl_inner_size();
    let cursor_position = frame.cursor_position();
    let inputs_pressed = frame.inputs_pressed();
    let inputs_released = frame.inputs_released();

    let len = value_mut.len() as f32;
    let width = f32::max(0.0, parent_size.x - 2.0 * theme.int_slider_margin);
    let label_width = LABEL_WIDTH_RATIO * width;
    let inner_width = f32::max(
        0.0,
        (width - label_width - LABEL_SPACING - INPUT_SPACING * (len - 1.0)) / len,
    );

    let mut outer_ctrl = frame.push_ctrl(id);
    outer_ctrl.set_flags(CtrlFlags::NONE);
    // TODO(yan): There's a TODO in ui layout that will allow us to put
    // horizontal layout here, but for now we do the layout by ourselves and
    // position both inner controls manually.
    outer_ctrl.set_layout(Layout::Free);
    outer_ctrl.set_rect(Rect::new(0.0, 0.0, width, theme.int_slider_height));
    outer_ctrl.set_padding(0.0);
    outer_ctrl.set_border(0.0);
    outer_ctrl.set_margin(theme.int_slider_margin);

    outer_ctrl.set_draw_self(false);
    outer_ctrl.draw_text_fitted(
        label,
        Align::Start,
        Align::Center,
        Wrap::Word,
        theme.int_slider_text_color,
        Rect::new(0.0, 0.0, label_width, theme.int_slider_height),
    );

    let mut changed = false;
    for (i, value_mut_slot) in value_mut.iter_mut().enumerate() {
        let mut inner_ctrl = frame.push_ctrl(cast_u32(i));
        inner_ctrl.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl.set_layout(Layout::Vertical);
        inner_ctrl.set_rect(Rect::new(
            label_width + LABEL_SPACING + (inner_width + INPUT_SPACING) * i as f32,
            0.0,
            inner_width,
            theme.int_slider_height,
        ));
        inner_ctrl.set_padding(0.0);
        inner_ctrl.set_border(theme.int_slider_border);
        inner_ctrl.set_margin(0.0);

        let hovered = inner_ctrl.is_hovered();
        let active = inner_ctrl.is_active();
        let state = cast_state(inner_ctrl.state());

        let (active, changed_i) = if active {
            let value = state.value;
            let x = state.x;
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl.set_active(false);
                false
            } else {
                true
            };

            let old_value = *value_mut_slot;
            let new_value = i32::min(
                i32::max(libm::roundf(value as f32 + delta * speed) as i32, min),
                max,
            );

            *value_mut_slot = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl.set_active(true);

            let state = cast_state_mut(inner_ctrl.state_mut());
            state.x = cursor_position.x;
            state.value = *value_mut_slot;

            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl.request_want_capture_keyboard();
        }

        changed |= changed_i;

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                theme.int_slider_text_color,
                theme.int_slider_background_color,
                theme.int_slider_border_color,
            ),
            (true, false) => (
                theme.int_slider_text_color_hovered,
                theme.int_slider_background_color_hovered,
                theme.int_slider_border_color_hovered,
            ),
            (_, true) => (
                theme.int_slider_text_color_active,
                theme.int_slider_background_color_active,
                theme.int_slider_border_color_active,
            ),
        };

        inner_ctrl.set_draw_self(true);
        inner_ctrl.set_draw_self_border_color(border_color);
        inner_ctrl.set_draw_self_background_color(background_color);

        s.clear();
        let _ = write!(s, "{value_mut_slot}");
        inner_ctrl.draw_text(&s, Align::Center, Align::Center, Wrap::Word, text_color);

        frame.pop_ctrl();
    }

    frame.pop_ctrl();

    changed
}

#[repr(C)]
#[derive(Clone, Copy)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
struct State {
    x: f32,
    value: i32,
}

fn cast_state(state: &CtrlState) -> &State {
    bytemuck::from_bytes(&state[..mem::size_of::<State>()])
}

fn cast_state_mut(state: &mut CtrlState) -> &mut State {
    bytemuck::from_bytes_mut(&mut state[..mem::size_of::<State>()])
}
