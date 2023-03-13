use alloc::vec::Vec;
use core::alloc::Allocator;
use core::mem;
use core::ops::{Deref, Range};

use arrayvec::{ArrayString, ArrayVec};

use crate::convert::cast_u32;
use crate::core::{
    Align,
    Ctrl,
    CtrlFlags,
    CtrlState,
    Frame,
    Inputs,
    Layout,
    Modifiers,
    Rect,
    TextStorage,
    Vec2,
    Wrap,
};
use crate::widgets::button::button;
use crate::widgets::theme::Theme;

const LABEL_WIDTH_RATIO: f32 = 0.35;
const LABEL_SPACING: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextInputCallbackData {
    pub active: bool,
    pub changed: bool,
    pub action: TextInputAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextInputAction {
    None,
    Submit,
    Cancel,
}

#[inline]
pub fn text_input<T, A>(frame: &mut Frame<A>, id: u32, text: &mut T, label: &str) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
{
    do_text_input_and_file_taxes::<_, _, &str>(
        frame,
        id,
        text,
        label,
        None,
        None,
        &[],
        &Theme::DEFAULT,
    )
}

#[inline]
pub fn text_input_with_autocomplete<T, A, D>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    autocomplete: &[D],
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
    D: Deref<Target = str>,
{
    do_text_input_and_file_taxes(
        frame,
        id,
        text,
        label,
        None,
        None,
        autocomplete,
        &Theme::DEFAULT,
    )
}

#[inline]
pub fn text_input_with_theme<T, A>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    theme: &Theme,
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
{
    do_text_input_and_file_taxes::<_, _, &str>(frame, id, text, label, None, None, &[], theme)
}

#[inline]
pub fn text_input_with_autocomplete_theme<T, A, D>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    autocomplete: &[D],
    theme: &Theme,
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
    D: Deref<Target = str>,
{
    do_text_input_and_file_taxes(frame, id, text, label, None, None, autocomplete, theme)
}

#[inline]
pub fn text_input_with_callback<T, A, C>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    mut callback: C,
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
    C: FnMut(&TextInputCallbackData, &mut T),
{
    do_text_input_and_file_taxes::<_, _, &str>(
        frame,
        id,
        text,
        label,
        Some(&mut callback),
        None,
        &[],
        &Theme::DEFAULT,
    )
}

#[inline]
pub fn text_input_with_callback_autocomplete<T, A, C, D>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    mut callback: C,
    autocomplete: &[D],
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
    C: FnMut(&TextInputCallbackData, &mut T),
    D: Deref<Target = str>,
{
    do_text_input_and_file_taxes(
        frame,
        id,
        text,
        label,
        Some(&mut callback),
        None,
        autocomplete,
        &Theme::DEFAULT,
    )
}

#[inline]
pub fn text_input_with_callback_theme<T, A, C>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    mut callback: C,
    theme: &Theme,
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
    C: FnMut(&TextInputCallbackData, &mut T),
{
    do_text_input_and_file_taxes::<_, _, &str>(
        frame,
        id,
        text,
        label,
        Some(&mut callback),
        None,
        &[],
        theme,
    )
}

#[inline]
pub fn text_input_with_callback_autocomplete_theme<T, A, C, D>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    mut callback: C,
    autocomplete: &[D],
    theme: &Theme,
) -> bool
where
    T: TextStorage,
    A: Allocator + Clone,
    C: FnMut(&TextInputCallbackData, &mut T),
    D: Deref<Target = str>,
{
    do_text_input_and_file_taxes(
        frame,
        id,
        text,
        label,
        Some(&mut callback),
        None,
        autocomplete,
        theme,
    )
}

// TODO(yan): @Cleanup This is exported just that numeric inputs can pass their
// filter-maps, but later we shuold expose this for everyone?
#[allow(clippy::type_complexity)]
pub(crate) fn do_text_input_and_file_taxes<T, A, D>(
    frame: &mut Frame<A>,
    id: u32,
    text: &mut T,
    label: &str,
    result_callback: Option<&mut dyn FnMut(&TextInputCallbackData, &mut T)>,
    filter_map_callback: Option<&dyn Fn(char) -> Option<char>>,
    autocomplete: &[D],
    theme: &Theme,
) -> bool
where
    A: Allocator + Clone,
    T: TextStorage,
    D: Deref<Target = str>,
{
    let parent_size = frame.ctrl_inner_size();
    let inputs_pressed = frame.inputs_pressed();
    let modifiers = frame.modifiers();

    let received_characters_unfiltered_count = frame.received_characters().len();
    let mut received_characters: ArrayString<32> = ArrayString::new();

    if let Some(fmc) = filter_map_callback {
        for c in frame.received_characters().chars() {
            if let Some(c) = fmc(c) {
                received_characters.push(c);
            }
        }
    } else {
        received_characters.push_str(frame.received_characters());
    }

    let outer_width = f32::max(0.0, parent_size.x - 2.0 * theme.text_input_margin);
    let label_width = LABEL_WIDTH_RATIO * outer_width;
    let inner_width = f32::max(0.0, outer_width - label_width - LABEL_SPACING);

    let mut outer_ctrl = frame.push_ctrl(id);
    outer_ctrl.set_flags(CtrlFlags::NONE);
    outer_ctrl.set_layout(Layout::Horizontal);
    outer_ctrl.set_rect(Rect::new(0.0, 0.0, outer_width, theme.text_input_height));
    outer_ctrl.set_padding(0.0);
    outer_ctrl.set_border(0.0);
    outer_ctrl.set_margin(theme.text_input_margin);

    outer_ctrl.set_draw_self(false);
    outer_ctrl.draw_text_fitted(
        label,
        Align::Start,
        Align::Center,
        Wrap::Word,
        theme.text_input_text_color,
        Rect::new(0.0, 0.0, label_width, theme.text_input_height),
    );

    let mut inner_ctrl = frame.push_ctrl(0);
    inner_ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER);
    inner_ctrl.set_layout(Layout::Vertical);
    inner_ctrl.set_rect(Rect::new(
        label_width + LABEL_SPACING,
        0.0,
        inner_width,
        theme.text_input_height,
    ));
    inner_ctrl.set_padding(0.0);
    inner_ctrl.set_border(theme.text_input_border);
    inner_ctrl.set_margin(0.0);

    let hovered = inner_ctrl.is_hovered();
    let active_orig = inner_ctrl.is_active();

    let state = cast_state(inner_ctrl.state());
    let mut text_cursor = usize::clamp(state.text_cursor, 0, text.len());
    let mut text_selection_start = usize::clamp(state.text_selection_start, 0, text.len());
    let mut text_selection_end = usize::clamp(state.text_selection_end, 0, text.len());
    let autocomplete_open = state.autocomplete_open;

    let mut deactivated_from_kb = false;

    let (active, changed, action) = if active_orig
        && (received_characters_unfiltered_count > 0 || inputs_pressed != Inputs::NONE)
    {
        let (handled, active, changed, action) = match inputs_pressed {
            Inputs::KB_BACKSPACE => {
                if text.len() > 0 {
                    let start = usize::min(text_selection_start, text_selection_end);
                    let end = usize::max(text_selection_start, text_selection_end);

                    if start != end {
                        // Ok to unwrap, because we are only removing.
                        text.try_splice(start, end - start, "").unwrap();

                        text_cursor = start;
                        text_selection_start = start;
                        text_selection_end = start;
                    } else if text_cursor == text.len() {
                        let text_cursor_after_trunc = seek_prev(text_cursor, text);

                        text.truncate(text_cursor_after_trunc);

                        text_cursor = text_cursor_after_trunc;
                        text_selection_start = text_cursor;
                        text_selection_end = text_cursor;
                    } else if text_cursor > 0 {
                        let text_cursor_after = seek_prev(text_cursor, text);
                        let delete_count = text_cursor - text_cursor_after;

                        // Ok to unwrap, because we are only removing.
                        text.try_splice(text_cursor_after, delete_count, "")
                            .unwrap();

                        text_cursor = text_cursor_after;
                        text_selection_start = text_cursor;
                        text_selection_end = text_cursor;
                    }

                    (true, true, true, TextInputAction::None)
                } else {
                    (true, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_DELETE => {
                if text.len() > 0 {
                    let last_char_index = seek_prev(text.len(), text);

                    if text_selection_start != text_selection_end {
                        let start = usize::min(text_selection_start, text_selection_end);
                        let end = usize::max(text_selection_start, text_selection_end);

                        // Ok to unwrap, because we are only removing.
                        text.try_splice(start, end - start, "").unwrap();

                        text_cursor = start;
                        text_selection_start = text_cursor;
                        text_selection_end = text_cursor;
                    } else if text_cursor == last_char_index {
                        text.truncate(last_char_index);

                        text_selection_start = text_cursor;
                        text_selection_end = text_cursor;
                    } else if text_cursor < last_char_index {
                        let delete_count = seek_next(text_cursor, text) - text_cursor;

                        // Ok to unwrap, because we are only removing.
                        text.try_splice(text_cursor, delete_count, "").unwrap();

                        text_selection_start = text_cursor;
                        text_selection_end = text_cursor;
                    }

                    (true, true, true, TextInputAction::None)
                } else {
                    (true, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_A => {
                if modifiers == Modifiers::CTRL {
                    text_cursor = 0;
                    text_selection_start = 0;
                    text_selection_end = text.len();

                    (true, true, false, TextInputAction::None)
                } else {
                    (false, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_LEFT_ARROW => {
                text_cursor = seek_prev(text_cursor, text);
                text_selection_end = text_cursor;
                if !modifiers.intersects(Modifiers::SHIFT) {
                    text_selection_start = text_cursor;
                }

                (true, true, false, TextInputAction::None)
            }

            Inputs::KB_B => {
                if modifiers == Modifiers::CTRL {
                    text_cursor = seek_prev(text_cursor, text);
                    text_selection_start = text_cursor;
                    text_selection_end = text_cursor;

                    (true, true, false, TextInputAction::None)
                } else if modifiers == Modifiers::CTRL | Modifiers::SHIFT {
                    text_cursor = seek_prev(text_cursor, text);
                    text_selection_end = text_cursor;

                    (true, true, false, TextInputAction::None)
                } else {
                    (false, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_RIGHT_ARROW => {
                text_cursor = seek_next(text_cursor, text);
                text_selection_end = text_cursor;
                if !modifiers.intersects(Modifiers::SHIFT) {
                    text_selection_start = text_cursor;
                }

                (true, true, false, TextInputAction::None)
            }

            Inputs::KB_F => {
                if modifiers == Modifiers::CTRL {
                    text_cursor = seek_next(text_cursor, text);
                    text_selection_start = text_cursor;
                    text_selection_end = text_cursor;

                    (true, true, false, TextInputAction::None)
                } else if modifiers == Modifiers::CTRL | Modifiers::SHIFT {
                    text_cursor = seek_next(text_cursor, text);
                    text_selection_end = text_cursor;

                    (true, true, false, TextInputAction::None)
                } else {
                    (false, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_X => {
                if modifiers == Modifiers::CTRL {
                    if text_selection_start != text_selection_end {
                        let start = usize::min(text_selection_start, text_selection_end);
                        let end = usize::max(text_selection_start, text_selection_end);

                        let s = &text[start..end];
                        inner_ctrl.set_clipboard_text(s);

                        text.try_splice(start, end - start, "").unwrap();

                        text_cursor = start;
                        text_selection_start = text_cursor;
                        text_selection_end = text_cursor;

                        (true, true, false, TextInputAction::None)
                    } else {
                        (true, true, false, TextInputAction::None)
                    }
                } else {
                    (false, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_C => {
                if modifiers == Modifiers::CTRL {
                    if text_selection_start != text_selection_end {
                        let start = usize::min(text_selection_start, text_selection_end);
                        let end = usize::max(text_selection_start, text_selection_end);

                        let s = &text[start..end];
                        inner_ctrl.set_clipboard_text(s);

                        (true, true, false, TextInputAction::None)
                    } else {
                        (true, true, false, TextInputAction::None)
                    }
                } else {
                    (false, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_V => {
                if modifiers == Modifiers::CTRL {
                    // start and end can be the same index here, in which
                    // case the splice will not remove anything, only insert
                    // stuff from the clipboard. If they are not the same,
                    // the selected text gets replaced.
                    let start = usize::min(text_selection_start, text_selection_end);
                    let end = usize::max(text_selection_start, text_selection_end);

                    let s = inner_ctrl.get_clipboard_text();
                    let _ = text.try_splice(start, end - start, &s);

                    text_cursor += s.len();
                    text_selection_start = text_cursor;
                    text_selection_end = text_cursor;

                    (true, true, false, TextInputAction::None)
                } else {
                    (false, true, false, TextInputAction::None)
                }
            }

            Inputs::KB_ENTER => {
                inner_ctrl.set_active(false);
                deactivated_from_kb = true;

                (true, false, false, TextInputAction::Submit)
            }

            Inputs::KB_ESCAPE => {
                inner_ctrl.set_active(false);
                deactivated_from_kb = true;

                (true, false, false, TextInputAction::Cancel)
            }

            _ => (false, true, false, TextInputAction::None),
        };

        if handled {
            (active, changed, action)
        } else {
            // TODO(yan): @Correctness If we missed frames, this structure
            // of handling inputs drops inputs received characters. Oh well.
            if text_selection_start != text_selection_end {
                let start = usize::min(text_selection_start, text_selection_end);
                let end = usize::max(text_selection_start, text_selection_end);

                let _ = text.try_splice(start, end - start, &received_characters);

                text_cursor = start + received_characters.len();
                text_selection_start = text_cursor;
                text_selection_end = text_cursor;
            } else if text_cursor == text.len() {
                let _ = text.try_extend(&received_characters);

                text_cursor = text.len();
                text_selection_start = text_cursor;
                text_selection_end = text_cursor;
            } else {
                let _ = text.try_splice(text_cursor, 0, &received_characters);

                text_cursor += received_characters.len();
                text_selection_start = text_cursor;
                text_selection_end = text_cursor;
            }

            (true, true, TextInputAction::None)
        }
    } else if hovered && inputs_pressed == Inputs::MB_LEFT {
        inner_ctrl.set_active(true);
        text_cursor = text.len();
        text_selection_start = text_cursor;
        text_selection_end = text_cursor;

        (true, false, TextInputAction::None)
    } else {
        (active_orig, false, TextInputAction::None)
    };

    let mut state = cast_state_mut(inner_ctrl.state_mut());
    state.text_cursor = text_cursor;
    state.text_selection_start = text_selection_start;
    state.text_selection_end = text_selection_end;
    if active {
        state.autocomplete_open = AUTOCOMPLETE_OPEN;
    }

    if active {
        inner_ctrl.request_want_capture_keyboard();
    }

    if let Some(result_callback) = result_callback {
        result_callback(
            &TextInputCallbackData {
                active,
                changed,
                action,
            },
            text,
        );
    }

    let (text_color, background_color, border_color) = match (hovered, active) {
        (false, false) => (
            theme.text_input_text_color,
            theme.text_input_background_color,
            theme.text_input_border_color,
        ),
        (true, false) => (
            theme.text_input_text_color_hovered,
            theme.text_input_background_color_hovered,
            theme.text_input_border_color_hovered,
        ),
        (_, true) => (
            theme.text_input_text_color_active,
            theme.text_input_background_color_active,
            theme.text_input_border_color_active,
        ),
    };

    inner_ctrl.set_draw_self(true);
    inner_ctrl.set_draw_self_border_color(border_color);
    inner_ctrl.set_draw_self_background_color(background_color);

    if active {
        draw(
            &mut inner_ctrl,
            text,
            Align::Center,
            Align::Center,
            text_color,
        );
    } else {
        inner_ctrl.draw_text(text, Align::Center, Align::Center, Wrap::None, text_color);
    }

    let mut changed_from_autocomplete = false;
    if autocomplete_open == AUTOCOMPLETE_OPEN && autocomplete.len() > 0 {
        let mut results: ArrayVec<&str, 20> = ArrayVec::new();

        if text.len() > 0 {
            // TODO(yan): Ignore case (but don't allocate!).
            // TODO(yan): Fuzzy string matching and sorting by score.
            let text_str: &str = text.deref();
            for candidate in autocomplete {
                let candidate_str: &str = candidate.deref();

                if candidate_str.contains(text_str) {
                    results.push(candidate_str);
                }

                if results.is_full() {
                    break;
                }
            }
        }

        if results.len() > 0 {
            let overlay_rect = {
                const OVERLAY_SPACING: f32 = 5.0;

                let absolute_position = inner_ctrl.absolute_position();

                let window_size = frame.window_size();
                let overlay_y = absolute_position.y + theme.text_input_height + OVERLAY_SPACING;

                let available_height_up = overlay_y;
                let available_height_down = f32::max(window_size.y - overlay_y, 0.0);

                let overlay_height_requested = f32::min(
                    results.len() as f32 * (theme.button_height + 2.0 * theme.button_margin),
                    theme.text_input_overlay_max_height,
                );

                if overlay_height_requested > available_height_down {
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
                }
            };

            frame.begin_overlay();

            let mut ctrl = frame.push_ctrl(id);
            ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER);
            ctrl.set_layout(Layout::Vertical);
            ctrl.set_rect(overlay_rect);

            // Margin is zero, because we are setting an absolute position.
            ctrl.set_padding(0.0);
            ctrl.set_border(theme.text_input_border);
            ctrl.set_margin(0.0);

            ctrl.set_draw_self(true);
            ctrl.set_draw_self_border_color(theme.text_input_border_color_active);
            ctrl.set_draw_self_background_color(theme.text_input_background_color_active);

            for (i, result) in results.into_iter().enumerate() {
                if button(frame, cast_u32(i), result) {
                    text.truncate(0);
                    let _ = text.try_extend(result);

                    changed_from_autocomplete = true;
                }
            }

            frame.pop_ctrl();

            frame.end_overlay();
        }
    }

    // TODO(yan): @Cleanup @Hack We have to track the open state of our
    // autocomplete dropdown manually, because we can't rely on us being active
    // after we render the overlay with the autcomplete choices, as those
    // buttons can take away the focus from us. the better way of doing this may
    // be to tell the button we don't want it to steal focus from us, in which
    // case we could rely on our own active state. This would help dropdown too.
    if changed_from_autocomplete || deactivated_from_kb {
        let state = cast_state_mut(frame.ctrl_state_mut());
        state.autocomplete_open = AUTOCOMPLETE_CLOSED;
    }

    frame.pop_ctrl();
    frame.pop_ctrl();

    changed || changed_from_autocomplete
}

const AUTOCOMPLETE_CLOSED: u32 = 0;
const AUTOCOMPLETE_OPEN: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
struct State {
    text_cursor: usize,
    text_selection_start: usize,
    text_selection_end: usize,
    autocomplete_open: u32,
    _pad0: u32,
}

fn cast_state(state: &CtrlState) -> &State {
    bytemuck::from_bytes(&state[..mem::size_of::<State>()])
}

fn cast_state_mut(state: &mut CtrlState) -> &mut State {
    bytemuck::from_bytes_mut(&mut state[..mem::size_of::<State>()])
}

// This is a modified text drawing routine from ui.rs. It doesn't handle
// word-wrapping and trimming, but can instead draw the cursor, text selection,
// handle horizontal and vertical scrolling within the text input, etc.
fn draw<A: Allocator + Clone>(
    ctrl: &mut Ctrl<A>,
    text: &str,
    halign: Align,
    valign: Align,
    color: u32,
) {
    let state = cast_state(ctrl.state());
    let text_cursor = state.text_cursor;
    let text_selection_start = usize::min(state.text_selection_start, state.text_selection_end);
    let text_selection_end = usize::max(state.text_selection_start, state.text_selection_end);

    let available_size = ctrl.inner_size();
    let available_width = available_size.x;
    let available_height = available_size.y;

    let font_atlas = ctrl.font_atlas();
    let font_atlas_texture_id = ctrl.font_atlas_texture_id();
    let font_size = font_atlas.font_size();

    struct Line {
        range: Range<usize>,
        width: f32,
    }

    // TODO(yan): @Memory If the allocator is a bump allocator, we
    // potentially prevent it from reclaiming memory if draw_primitives
    // grow.
    let mut lines: Vec<Line, _> = Vec::new_in(ctrl.allocator().clone());

    let mut line_range = 0..0;
    let mut line_width = 0.0;

    for (i, c) in text.char_indices() {
        if c == '\n' && !line_range.is_empty() {
            // Note that this could be an empty line, but that's fine.
            lines.push(Line {
                range: line_range,
                width: line_width,
            });

            // 1 is the byte width of the '\n', so i + 1 is ok.
            line_range = i + 1..i + 1;
            line_width = 0.0;

            continue;
        }

        let glyph_info = font_atlas.glyph_info(c);
        let glyph_advance_width = glyph_info.advance_width;

        line_range.end += c.len_utf8();
        line_width += glyph_advance_width;
    }

    lines.push(Line {
        range: line_range,
        width: line_width,
    });

    //
    // Emit rects based on generated line data.
    //
    let line_metrics = font_atlas.font_horizontal_line_metrics();

    let mut position_x = 0.0;
    let mut position_y = if lines.len() as f32 * line_metrics.new_line_size < available_height {
        match valign {
            Align::Start => line_metrics.line_gap,
            Align::Center => {
                let line_gap = line_metrics.line_gap;
                let new_line_size = line_metrics.new_line_size;
                let text_block_size = new_line_size * lines.len() as f32 - line_gap;

                line_gap + (available_height - text_block_size) / 2.0
            }
            Align::End => {
                let line_gap = line_metrics.line_gap;
                let new_line_size = line_metrics.new_line_size;
                let text_block_size = new_line_size * lines.len() as f32 - line_gap;

                line_gap + available_height - text_block_size
            }
        }
    } else {
        line_metrics.line_gap
    };

    let mut cursor_drawn = false;
    let mut selection_rect = Rect::ZERO;

    for line in &lines {
        let line_slice = &text[line.range.clone()];

        position_x = match halign {
            Align::Start => 0.0,
            Align::Center => (available_width - line.width) / 2.0,
            Align::End => available_width - line.width,
        };

        for (i, c) in line_slice.chars().enumerate() {
            // Reborrow font_atlas, so that the globally borrowed one is
            // released and we can call Ctrl::draw_rect.
            let font_atlas = ctrl.font_atlas();
            let glyph_info = font_atlas.glyph_info(c);

            let position = Vec2::new(position_x, position_y);
            let rect = glyph_info.rect + position + Vec2::y(line_metrics.ascent);

            let text_position = i + line.range.start;
            if text_position == text_cursor {
                ctrl.draw_rect(
                    Rect::new(
                        position_x,
                        position_y,
                        1.0,
                        line_metrics.ascent - line_metrics.descent,
                    ),
                    Rect::ZERO,
                    0x40ffa0c0,
                    font_atlas_texture_id,
                );
                cursor_drawn = true;
            }

            if text_position >= text_selection_start && text_position <= text_selection_end {
                let r = Rect::new(
                    position.x,
                    position_y,
                    0.0,
                    line_metrics.ascent - line_metrics.descent,
                );

                if selection_rect == Rect::ZERO {
                    selection_rect = r;
                } else {
                    selection_rect = selection_rect.extend_by_rect(r);
                }
            }

            // TODO(yan): @Speed @Memory Does early software scissor make
            // sense here? We also do it later, when translating to the
            // low-level draw list, but we could have less things to
            // translate.
            ctrl.draw_rect(rect, glyph_info.atlas_rect, color, font_atlas_texture_id);

            position_x += glyph_info.advance_width;
        }

        position_y += line_metrics.new_line_size;
    }

    if selection_rect != Rect::ZERO {
        if text_selection_end == text.len() {
            selection_rect = selection_rect.extend_by_point(Vec2::new(position_x, position_y));
        }

        ctrl.draw_rect(
            selection_rect,
            Rect::ZERO,
            0x40ffa040,
            font_atlas_texture_id,
        )
    }

    if !cursor_drawn {
        let rect = Rect::new(
            position_x,
            position_y - line_metrics.ascent + line_metrics.descent,
            font_size / 2.0,
            line_metrics.ascent - line_metrics.descent,
        );

        ctrl.draw_rect(rect, Rect::ZERO, 0x40ffa0c0, font_atlas_texture_id);
    }
}

fn seek_prev(index: usize, text: &str) -> usize {
    debug_assert!(index <= text.len());
    text.floor_char_boundary(index.saturating_sub(1))
}

fn seek_next(index: usize, text: &str) -> usize {
    debug_assert!(index <= text.len());

    // Cursor can point at one past last index.
    if index < text.len() {
        text.ceil_char_boundary(index + 1)
    } else {
        index
    }
}
