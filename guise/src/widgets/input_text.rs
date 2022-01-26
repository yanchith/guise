use core::alloc::Allocator;
use core::ops::Deref;

use arrayvec::ArrayString;

use crate::core::{Align, CtrlFlags, Frame, Inputs, Layout, Rect, Vec2, Wrap};
use crate::widgets::theme::Theme;

// TODO(yan): This needs features. A lot of features. Arrow key and
// mouse movement, copy/paste, undo, etc.

// TODO(yan): Implement feature-flagged EditableText for popular smallstring
// libraries, where applicable.
//
// TODO(yan): Reuse this in all text components.
pub trait EditableText: Deref<Target = str> {
    fn push(&mut self, c: char);
    fn pop(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputTextSubmit {
    None,
    Submit,
    Cancel,
}

pub fn input_text<T, A, TA>(
    frame: &mut Frame<A, TA>,
    id: u32,
    text: &mut T,
) -> (bool, InputTextSubmit)
where
    T: EditableText,
    A: Allocator + Clone,
    TA: Allocator,
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
    T: EditableText,
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

    pub fn show<A, TA>(&mut self, frame: &mut Frame<A, TA>) -> (bool, InputTextSubmit)
    where
        A: Allocator + Clone,
        TA: Allocator,
    {
        let parent_size = frame.ctrl_inner_size();
        let inputs_pressed = frame.inputs_pressed();
        let received_characters: ArrayString<32> =
            ArrayString::from(frame.received_characters()).unwrap();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.input_text_margin);

        let mut ctrl = frame.push_ctrl(self.id);
        ctrl.set_flags(CtrlFlags::CAPTURE_SCROLL | CtrlFlags::CAPTURE_HOVER);
        ctrl.set_layout(Layout::Vertical);
        ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.input_text_height));
        ctrl.set_padding(self.theme.input_text_padding);
        ctrl.set_border(self.theme.input_text_border);
        ctrl.set_margin(self.theme.input_text_margin);

        let hovered = ctrl.hovered();
        let active = ctrl.active();

        let (active, changed, submit) =
            if active && (!received_characters.is_empty() || inputs_pressed != Inputs::NONE) {
                if inputs_pressed != Inputs::NONE {
                    match inputs_pressed {
                        Inputs::KEYBOARD_BACKSPACE => {
                            self.text.pop();
                            (true, true, InputTextSubmit::None)
                        }
                        Inputs::KEYBOARD_ENTER => {
                            ctrl.set_active(false);
                            (false, false, InputTextSubmit::Submit)
                        }
                        Inputs::KEYBOARD_ESCAPE => {
                            ctrl.set_active(false);
                            (false, false, InputTextSubmit::Cancel)
                        }
                        _ => (true, false, InputTextSubmit::None),
                    }
                } else {
                    // TODO(yan): This likely won't be robust enough for
                    // multiple chars per frame. We should control chars like
                    // backspace, delete, enter here, but because we process
                    // Inputs first, we never get here with special chars.
                    for c in received_characters.chars() {
                        self.text.push(c);
                    }

                    (true, true, InputTextSubmit::None)
                }
            } else if hovered && inputs_pressed == Inputs::MOUSE_BUTTON_LEFT {
                ctrl.set_active(true);
                (true, false, InputTextSubmit::None)
            } else {
                (active, false, InputTextSubmit::None)
            };

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
        ctrl.draw_text_ex(
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
}

impl<const C: usize> EditableText for arrayvec::ArrayString<C> {
    fn push(&mut self, c: char) {
        let _ = self.try_push(c);
    }

    fn pop(&mut self) {
        self.pop();
    }
}
