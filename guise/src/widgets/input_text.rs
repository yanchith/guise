use core::alloc::Allocator;

use arrayvec::ArrayString;

use crate::convert::{cast_u32, cast_usize};
use crate::core::{Align, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, TextStorage, Wrap};
use crate::widgets::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputTextSubmit {
    None,
    Submit,
    Cancel,
}

pub fn input_text<T, A>(frame: &mut Frame<A>, id: u32, text: &mut T) -> (bool, InputTextSubmit)
where
    T: TextStorage,
    A: Allocator + Clone,
{
    InputText::new(id, text).show(frame)
}

pub struct InputText<'a, T> {
    id: u32,
    text: &'a mut T,
    theme: &'a Theme,
}

impl<'a, T> InputText<'a, T>
where
    T: TextStorage,
{
    pub fn new(id: u32, text: &'a mut T) -> Self {
        Self {
            id,
            text,
            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&mut self, frame: &mut Frame<A>) -> (bool, InputTextSubmit) {
        let parent_size = frame.ctrl_inner_size();
        let inputs_pressed = frame.inputs_pressed();
        let received_characters: ArrayString<32> =
            ArrayString::from(frame.received_characters()).unwrap();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.input_text_margin);
        let border = self.theme.input_text_border;
        let padding = self.theme.input_text_padding;

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.input_text_height));
        ctrl.set_padding(padding);
        ctrl.set_border(border);
        ctrl.set_margin(self.theme.input_text_margin);

        let hovered = ctrl.hovered();
        let active = ctrl.active();

        let mut text_cursor = text_cursor(ctrl.state());
        text_cursor = u32::clamp(text_cursor, 0, cast_u32(self.text.len()));

        let (active, changed, submit) =
            if active && (!received_characters.is_empty() || inputs_pressed != Inputs::NONE) {
                if inputs_pressed != Inputs::NONE {
                    let text_len_u32 = cast_u32(self.text.len());

                    match inputs_pressed {
                        Inputs::KB_BACKSPACE => {
                            if self.text.len() > 0 {
                                if text_cursor == text_len_u32 {
                                    self.text.truncate(self.text.len() - 1);
                                    text_cursor -= 1;
                                } else {
                                    debug_assert!(text_cursor < text_len_u32);
                                    if text_cursor > 0 {
                                        // NB: Ok to unwrap, we are only removing.
                                        self.text
                                            .try_splice(cast_usize(text_cursor - 1), 1, "")
                                            .unwrap();
                                        text_cursor -= 1;
                                    }
                                }

                                (true, true, InputTextSubmit::None)
                            } else {
                                (true, false, InputTextSubmit::None)
                            }
                        }

                        Inputs::KB_DELETE => {
                            if self.text.len() > 0 {
                                if text_cursor == text_len_u32 - 1 {
                                    self.text.truncate(self.text.len() - 1);
                                } else if text_cursor < text_len_u32 - 1 {
                                    self.text
                                        .try_splice(cast_usize(text_cursor), 1, "")
                                        .unwrap();
                                }
                                (true, true, InputTextSubmit::None)
                            } else {
                                (true, false, InputTextSubmit::None)
                            }
                        }

                        Inputs::KB_LEFT_ARROW => {
                            if text_cursor > 0 {
                                text_cursor -= 1;
                            }

                            (true, false, InputTextSubmit::None)
                        }

                        Inputs::KB_RIGHT_ARROW => {
                            if text_cursor < text_len_u32 {
                                text_cursor += 1;
                            }

                            (true, false, InputTextSubmit::None)
                        }

                        Inputs::KB_ENTER => {
                            ctrl.set_active(false);
                            (false, false, InputTextSubmit::Submit)
                        }

                        Inputs::KB_ESCAPE => {
                            ctrl.set_active(false);
                            (false, false, InputTextSubmit::Cancel)
                        }

                        _ => (true, false, InputTextSubmit::None),
                    }
                } else {
                    // TODO(yan): This likely won't be robust enough for
                    // multiple chars per frame. We should control chars like
                    // backspace, delete, enter here, but because we process
                    // Inputs in the other branch, we never get here with
                    // special chars.
                    if text_cursor == cast_u32(self.text.len()) {
                        let _ = self.text.try_extend(&received_characters);

                        text_cursor = cast_u32(self.text.len());
                    } else {
                        let p = cast_usize(text_cursor);
                        let _ = self.text.try_splice(p, 0, &received_characters);

                        // NB: Text cursor operates on characters, so we have to
                        // count them and not use the byte length.
                        text_cursor += cast_u32(received_characters.chars().count());
                    }

                    (true, true, InputTextSubmit::None)
                }
            } else if hovered && inputs_pressed == Inputs::MB_LEFT {
                ctrl.set_active(true);
                (true, false, InputTextSubmit::None)
            } else {
                (active, false, InputTextSubmit::None)
            };

        set_text_cursor(ctrl.state_mut(), text_cursor);

        if active {
            ctrl.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.input_text_text_color,
                self.theme.input_text_background_color,
                self.theme.input_text_border_color,
            ),
            (true, false) => (
                self.theme.input_text_text_color_hovered,
                self.theme.input_text_background_color_hovered,
                self.theme.input_text_border_color_hovered,
            ),
            (_, true) => (
                self.theme.input_text_text_color_active,
                self.theme.input_text_background_color_active,
                self.theme.input_text_border_color_active,
            ),
        };

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(border_color);
        ctrl.set_draw_self_background_color(background_color);

        // TODO(yan): The text cursor should always be on screen. This requires
        // text layout to happen first.
        ctrl.draw_text(
            true,
            None,
            border + padding,
            self.text,
            Align::Start,
            Align::Center,
            Wrap::None,
            text_color,
        );

        frame.pop_ctrl();

        (changed, submit)
    }
}

fn text_cursor(state: &CtrlState) -> u32 {
    u32::from_le_bytes([state[0], state[1], state[2], state[3]])
}

fn set_text_cursor(state: &mut CtrlState, text_cursor: u32) {
    let bytes = text_cursor.to_le_bytes();
    state[0] = bytes[0];
    state[1] = bytes[1];
    state[2] = bytes[2];
    state[3] = bytes[3];
}
