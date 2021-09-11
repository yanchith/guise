use core::ops::{Deref, DerefMut};

use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Vec2, Wrap};
use crate::widgets::theme::Theme;

use arrayvec::ArrayString;

// TODO(yan): This needs features. A lot of features. Arrow key and
// mouse movement, copy/paste, undo, etc.

// TODO(yan): Implement feature-flagged EditableText for popular smallstring
// libraries, where applicable.
//
// TODO(yan): Reuse this in all text components.
pub trait EditableText: Deref<Target = str> + DerefMut<Target = str> {
    fn push(&mut self, c: char);
    fn pop(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextInputSubmit {
    None,
    Submit,
    Cancel,
}

pub fn text_input<T>(
    frame: &mut Frame,
    id: u32,
    theme: &Theme,
    text: &mut T,
) -> (bool, TextInputSubmit)
where
    T: EditableText,
{
    TextInput::new(id, theme, text).show(frame)
}

pub struct TextInput<'a, T> {
    id: u32,
    theme: &'a Theme,
    text: &'a mut T,
}

impl<'a, T> TextInput<'a, T>
where
    T: EditableText,
{
    pub fn new(id: u32, theme: &'a Theme, text: &'a mut T) -> Self {
        Self { id, theme, text }
    }

    pub fn show(&mut self, frame: &mut Frame) -> (bool, TextInputSubmit) {
        let inputs_pressed = frame.window_inputs_pressed();
        let received_characters: ArrayString<32> =
            ArrayString::from(frame.window_received_characters()).unwrap();

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(
            0.0,
            0.0,
            self.theme.text_input_width,
            self.theme.text_input_height,
        ));
        ctrl.set_padding(self.theme.text_input_padding);
        ctrl.set_border(self.theme.text_input_border);
        ctrl.set_margin(self.theme.text_input_margin);

        let hovered = ctrl.hovered();
        let active = ctrl.active();

        let (active, changed, submit) =
            if active && (!received_characters.is_empty() || inputs_pressed != Inputs::NONE) {
                if inputs_pressed != Inputs::NONE {
                    match inputs_pressed {
                        Inputs::KEYBOARD_BACKSPACE => {
                            self.text.pop();
                            (true, true, TextInputSubmit::None)
                        }
                        Inputs::KEYBOARD_ENTER => {
                            ctrl.set_active(false);
                            (false, false, TextInputSubmit::Submit)
                        }
                        Inputs::KEYBOARD_ESCAPE => {
                            ctrl.set_active(false);
                            (false, false, TextInputSubmit::Cancel)
                        }
                        _ => (true, false, TextInputSubmit::None),
                    }
                } else {
                    // TODO(yan): This likely won't be robust enough for
                    // multiple chars per frame. We should control chars like
                    // backspace, delete, enter here, but because we process
                    // Inputs first, we never get here with special chars.
                    for c in received_characters.chars() {
                        self.text.push(c);
                    }

                    (true, true, TextInputSubmit::None)
                }
            } else if hovered && inputs_pressed == Inputs::MOUSE_BUTTON_LEFT {
                ctrl.set_active(true);
                (true, false, TextInputSubmit::None)
            } else {
                (active, false, TextInputSubmit::None)
            };

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.text_input_text_color,
                self.theme.text_input_background_color,
                self.theme.text_input_border_color,
            ),
            (true, false) => (
                self.theme.text_input_text_color_hovered,
                self.theme.text_input_background_color_hovered,
                self.theme.text_input_border_color_hovered,
            ),
            (_, true) => (
                self.theme.text_input_text_color_active,
                self.theme.text_input_background_color_active,
                self.theme.text_input_border_color_active,
            ),
        };

        ctrl.set_draw_self(true);
        ctrl.set_draw_self_border_color(border_color);
        ctrl.set_draw_self_background_color(background_color);
        ctrl.draw_text(
            true,
            Vec2::ZERO,
            self.text,
            Align::Start,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        (changed, submit)
    }
}

impl EditableText for alloc::string::String {
    fn push(&mut self, c: char) {
        self.push(c);
    }

    fn pop(&mut self) {
        self.pop();
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<const C: usize> EditableText for arrayvec::ArrayString<C> {
    fn push(&mut self, c: char) {
        let _ = self.try_push(c);
    }

    fn pop(&mut self) {
        self.pop();
    }

    fn len(&self) -> usize {
        self.len()
    }
}
