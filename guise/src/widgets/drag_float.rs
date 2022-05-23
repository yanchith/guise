use core::alloc::Allocator;
use core::fmt::Write;

use arrayvec::ArrayString;

use crate::core::{Align, CtrlFlags, CtrlState, Frame, Inputs, Layout, Rect, Wrap};
use crate::widgets::theme::Theme;

// TODO(yan): Do DragFloat4.

// TODO(yan): @Cleanup there's a lot of copypaste between the drag floats
// implementations. Perhaps we want to do something tiny bit more abstract, like
// a fn show() that takes a slice and based on the length of that loops and
// draws stuff.

const LABEL_WIDTH_RATIO: f32 = 0.4;
const LABEL_SPACING: f32 = 5.0;
const INPUT_SPACING: f32 = 2.0;

pub fn drag_float<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut f32,
    label: &str,
) -> bool {
    DragFloat::new(id, value, label).show(frame)
}

pub fn drag_float2<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [f32; 2],
    label: &str,
) -> bool {
    DragFloat2::new(id, value, label).show(frame)
}

pub fn drag_float3<A: Allocator + Clone>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut [f32; 3],
    label: &str,
) -> bool {
    DragFloat3::new(id, value, label).show(frame)
}

pub struct DragFloat<'a> {
    id: u32,
    value: &'a mut f32,
    label: &'a str,

    speed: f32,
    min: f32,
    max: f32,
    display_precision: u16,

    theme: &'a Theme,
}

impl<'a> DragFloat<'a> {
    pub fn new(id: u32, value: &'a mut f32, label: &'a str) -> Self {
        Self {
            id,
            value,
            label,

            speed: 1.0,
            min: f32::MIN,
            max: f32::MAX,
            display_precision: 3,

            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn set_min(&mut self, min: f32) -> &mut Self {
        self.min = min;
        self
    }

    pub fn set_max(&mut self, max: f32) -> &mut Self {
        self.max = max;
        self
    }

    pub fn set_display_precision(&mut self, display_precision: u16) -> &mut Self {
        self.display_precision = display_precision;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&mut self, frame: &mut Frame<A>) -> bool {
        let mut s: ArrayString<256> = ArrayString::new();

        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();
        let inputs_pressed = frame.inputs_pressed();
        let inputs_released = frame.inputs_released();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.drag_float_margin);
        let label_width = LABEL_WIDTH_RATIO * width;
        let inner_width = f32::max(0.0, width - label_width - LABEL_SPACING);

        let mut outer_ctrl = frame.push_ctrl(self.id);
        outer_ctrl.set_flags(CtrlFlags::NONE);
        outer_ctrl.set_layout(Layout::Horizontal);
        outer_ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.drag_float_height));
        outer_ctrl.set_padding(0.0);
        outer_ctrl.set_border(0.0);
        outer_ctrl.set_margin(self.theme.drag_float_margin);

        outer_ctrl.set_draw_self(false);
        outer_ctrl.draw_text(
            true,
            Some(Rect::new(
                0.0,
                0.0,
                label_width,
                self.theme.drag_float_height,
            )),
            0.0,
            self.label,
            Align::Start,
            Align::Center,
            Wrap::Word,
            self.theme.drag_float_text_color,
        );

        let mut inner_ctrl = frame.push_ctrl(0);
        inner_ctrl.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl.set_layout(Layout::Vertical);
        inner_ctrl.set_rect(Rect::new(
            label_width + LABEL_SPACING,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl.set_padding(0.0);
        inner_ctrl.set_border(self.theme.drag_float_border);
        inner_ctrl.set_margin(0.0);

        let hovered = inner_ctrl.hovered();
        let active = inner_ctrl.active();
        let state = inner_ctrl.state();

        let (active, changed) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl.set_active(false);
                false
            } else {
                true
            };

            let old_value = *self.value;
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            *self.value = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl.set_active(true);
            let state = inner_ctrl.state_mut();
            set_value(state, *self.value);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        inner_ctrl.set_draw_self(true);
        inner_ctrl.set_draw_self_border_color(border_color);
        inner_ctrl.set_draw_self_background_color(background_color);

        let _ = write!(
            s,
            "{:.1$}",
            *self.value,
            usize::from(self.display_precision)
        );
        inner_ctrl.draw_text(
            true,
            None,
            0.0,
            &s,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();
        frame.pop_ctrl();

        changed
    }
}

pub struct DragFloat2<'a> {
    id: u32,
    value: &'a mut [f32; 2],
    label: &'a str,

    speed: f32,
    min: f32,
    max: f32,
    display_precision: u16,

    theme: &'a Theme,
}

impl<'a> DragFloat2<'a> {
    pub fn new(id: u32, value: &'a mut [f32; 2], label: &'a str) -> Self {
        Self {
            id,
            value,
            label,

            speed: 1.0,
            min: f32::MIN,
            max: f32::MAX,
            display_precision: 3,

            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn set_min(&mut self, min: f32) -> &mut Self {
        self.min = min;
        self
    }

    pub fn set_max(&mut self, max: f32) -> &mut Self {
        self.max = max;
        self
    }

    pub fn set_display_precision(&mut self, display_precision: u16) -> &mut Self {
        self.display_precision = display_precision;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&mut self, frame: &mut Frame<A>) -> bool {
        let mut s: ArrayString<256> = ArrayString::new();

        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();
        let inputs_pressed = frame.inputs_pressed();
        let inputs_released = frame.inputs_released();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.drag_float_margin);
        let label_width = LABEL_WIDTH_RATIO * width;
        let inner_width = f32::max(
            0.0,
            (width - label_width - LABEL_SPACING - INPUT_SPACING) / 2.0,
        );

        let mut outer_ctrl = frame.push_ctrl(self.id);
        outer_ctrl.set_flags(CtrlFlags::NONE);
        // TODO(yan): There's a TODO in ui layout that will allow us to put
        // horizontal layout here, but for now we do the layout by ourselves and
        // position both inner controls manually.
        outer_ctrl.set_layout(Layout::Free);
        outer_ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.drag_float_height));
        outer_ctrl.set_padding(0.0);
        outer_ctrl.set_border(0.0);
        outer_ctrl.set_margin(self.theme.drag_float_margin);

        outer_ctrl.set_draw_self(false);
        outer_ctrl.draw_text(
            true,
            Some(Rect::new(
                0.0,
                0.0,
                label_width,
                self.theme.drag_float_height,
            )),
            0.0,
            self.label,
            Align::Start,
            Align::Center,
            Wrap::Word,
            self.theme.drag_float_text_color,
        );

        let mut inner_ctrl_x = frame.push_ctrl(0);
        inner_ctrl_x.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl_x.set_layout(Layout::Vertical);
        inner_ctrl_x.set_rect(Rect::new(
            label_width + LABEL_SPACING,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl_x.set_padding(0.0);
        inner_ctrl_x.set_border(self.theme.drag_float_border);
        inner_ctrl_x.set_margin(0.0);

        let hovered = inner_ctrl_x.hovered();
        let active = inner_ctrl_x.active();
        let state = inner_ctrl_x.state();

        let (active, changed_x) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl_x.set_active(false);
                false
            } else {
                true
            };

            let old_value = self.value[0];
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            self.value[0] = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl_x.set_active(true);
            let state = inner_ctrl_x.state_mut();
            set_value(state, self.value[0]);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl_x.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        inner_ctrl_x.set_draw_self(true);
        inner_ctrl_x.set_draw_self_border_color(border_color);
        inner_ctrl_x.set_draw_self_background_color(background_color);

        let _ = write!(
            s,
            "{:.1$}",
            self.value[0],
            usize::from(self.display_precision)
        );
        inner_ctrl_x.draw_text(
            true,
            None,
            0.0,
            &s,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        let mut inner_ctrl_y = frame.push_ctrl(1);
        inner_ctrl_y.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl_y.set_layout(Layout::Vertical);
        inner_ctrl_y.set_rect(Rect::new(
            label_width + LABEL_SPACING + inner_width + INPUT_SPACING,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl_y.set_padding(0.0);
        inner_ctrl_y.set_border(self.theme.drag_float_border);
        inner_ctrl_y.set_margin(0.0);

        let hovered = inner_ctrl_y.hovered();
        let active = inner_ctrl_y.active();
        let state = inner_ctrl_y.state();

        let (active, changed_y) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl_y.set_active(false);
                false
            } else {
                true
            };

            let old_value = self.value[1];
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            self.value[1] = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl_y.set_active(true);
            let state = inner_ctrl_y.state_mut();
            set_value(state, self.value[1]);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl_y.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        inner_ctrl_y.set_draw_self(true);
        inner_ctrl_y.set_draw_self_border_color(border_color);
        inner_ctrl_y.set_draw_self_background_color(background_color);

        s.clear();
        let _ = write!(
            s,
            "{:.1$}",
            self.value[1],
            usize::from(self.display_precision)
        );
        inner_ctrl_y.draw_text(
            true,
            None,
            0.0,
            &s,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();
        frame.pop_ctrl();

        changed_x | changed_y
    }
}

pub struct DragFloat3<'a> {
    id: u32,
    value: &'a mut [f32; 3],
    label: &'a str,

    speed: f32,
    min: f32,
    max: f32,
    display_precision: u16,

    theme: &'a Theme,
}

impl<'a> DragFloat3<'a> {
    pub fn new(id: u32, value: &'a mut [f32; 3], label: &'a str) -> Self {
        Self {
            id,
            value,
            label,

            speed: 1.0,
            min: f32::MIN,
            max: f32::MAX,
            display_precision: 3,

            theme: &Theme::DEFAULT,
        }
    }

    pub fn set_speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn set_min(&mut self, min: f32) -> &mut Self {
        self.min = min;
        self
    }

    pub fn set_max(&mut self, max: f32) -> &mut Self {
        self.max = max;
        self
    }

    pub fn set_display_precision(&mut self, display_precision: u16) -> &mut Self {
        self.display_precision = display_precision;
        self
    }

    pub fn set_theme(&mut self, theme: &'a Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn show<A: Allocator + Clone>(&mut self, frame: &mut Frame<A>) -> bool {
        let mut s: ArrayString<256> = ArrayString::new();

        let parent_size = frame.ctrl_inner_size();
        let cursor_position = frame.cursor_position();
        let inputs_pressed = frame.inputs_pressed();
        let inputs_released = frame.inputs_released();

        let width = f32::max(0.0, parent_size.x - 2.0 * self.theme.drag_float_margin);
        let label_width = LABEL_WIDTH_RATIO * width;
        let inner_width = f32::max(
            0.0,
            (width - label_width - LABEL_SPACING - INPUT_SPACING * 2.0) / 3.0,
        );

        let mut outer_ctrl = frame.push_ctrl(self.id);
        outer_ctrl.set_flags(CtrlFlags::NONE);
        // TODO(yan): There's a TODO in ui layout that will allow us to put
        // horizontal layout here, but for now we do the layout by ourselves and
        // position both inner controls manually.
        outer_ctrl.set_layout(Layout::Free);
        outer_ctrl.set_rect(Rect::new(0.0, 0.0, width, self.theme.drag_float_height));
        outer_ctrl.set_padding(0.0);
        outer_ctrl.set_border(0.0);
        outer_ctrl.set_margin(self.theme.drag_float_margin);

        outer_ctrl.set_draw_self(false);
        outer_ctrl.draw_text(
            true,
            Some(Rect::new(
                0.0,
                0.0,
                label_width,
                self.theme.drag_float_height,
            )),
            0.0,
            self.label,
            Align::Start,
            Align::Center,
            Wrap::Word,
            self.theme.drag_float_text_color,
        );

        let mut inner_ctrl_x = frame.push_ctrl(0);
        inner_ctrl_x.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl_x.set_layout(Layout::Vertical);
        inner_ctrl_x.set_rect(Rect::new(
            label_width + LABEL_SPACING,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl_x.set_padding(0.0);
        inner_ctrl_x.set_border(self.theme.drag_float_border);
        inner_ctrl_x.set_margin(0.0);

        let hovered = inner_ctrl_x.hovered();
        let active = inner_ctrl_x.active();
        let state = inner_ctrl_x.state();

        let (active, changed_x) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl_x.set_active(false);
                false
            } else {
                true
            };

            let old_value = self.value[0];
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            self.value[0] = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl_x.set_active(true);
            let state = inner_ctrl_x.state_mut();
            set_value(state, self.value[0]);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl_x.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        inner_ctrl_x.set_draw_self(true);
        inner_ctrl_x.set_draw_self_border_color(border_color);
        inner_ctrl_x.set_draw_self_background_color(background_color);

        let _ = write!(
            s,
            "{:.1$}",
            self.value[0],
            usize::from(self.display_precision)
        );
        inner_ctrl_x.draw_text(
            true,
            None,
            0.0,
            &s,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        let mut inner_ctrl_y = frame.push_ctrl(1);
        inner_ctrl_y.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl_y.set_layout(Layout::Vertical);
        inner_ctrl_y.set_rect(Rect::new(
            label_width + LABEL_SPACING + inner_width + INPUT_SPACING,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl_y.set_padding(0.0);
        inner_ctrl_y.set_border(self.theme.drag_float_border);
        inner_ctrl_y.set_margin(0.0);

        let hovered = inner_ctrl_y.hovered();
        let active = inner_ctrl_y.active();
        let state = inner_ctrl_y.state();

        let (active, changed_y) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl_y.set_active(false);
                false
            } else {
                true
            };

            let old_value = self.value[1];
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            self.value[1] = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl_y.set_active(true);
            let state = inner_ctrl_y.state_mut();
            set_value(state, self.value[1]);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl_y.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        inner_ctrl_y.set_draw_self(true);
        inner_ctrl_y.set_draw_self_border_color(border_color);
        inner_ctrl_y.set_draw_self_background_color(background_color);

        s.clear();
        let _ = write!(
            s,
            "{:.1$}",
            self.value[1],
            usize::from(self.display_precision)
        );
        inner_ctrl_y.draw_text(
            true,
            None,
            0.0,
            &s,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();

        let mut inner_ctrl_z = frame.push_ctrl(2);
        inner_ctrl_z.set_flags(CtrlFlags::CAPTURE_HOVER);
        inner_ctrl_z.set_layout(Layout::Vertical);
        inner_ctrl_z.set_rect(Rect::new(
            label_width + LABEL_SPACING + (inner_width + INPUT_SPACING) * 2.0,
            0.0,
            inner_width,
            self.theme.drag_float_height,
        ));
        inner_ctrl_z.set_padding(0.0);
        inner_ctrl_z.set_border(self.theme.drag_float_border);
        inner_ctrl_z.set_margin(0.0);

        let hovered = inner_ctrl_z.hovered();
        let active = inner_ctrl_z.active();
        let state = inner_ctrl_z.state();

        let (active, changed_z) = if active {
            let value = value(state);
            let x = x(state);
            let delta = cursor_position.x - x;

            let new_active = if inputs_released == Inputs::MB_LEFT {
                inner_ctrl_z.set_active(false);
                false
            } else {
                true
            };

            let old_value = self.value[2];
            let new_value = f32::clamp(value + delta * self.speed, self.min, self.max);

            self.value[2] = new_value;
            (new_active, old_value != new_value)
        } else if hovered && inputs_pressed == Inputs::MB_LEFT {
            inner_ctrl_z.set_active(true);
            let state = inner_ctrl_z.state_mut();
            set_value(state, self.value[2]);
            set_x(state, cursor_position.x);
            (true, false)
        } else {
            (active, false)
        };

        if active {
            inner_ctrl_z.request_want_capture_keyboard();
        }

        let (text_color, background_color, border_color) = match (hovered, active) {
            (false, false) => (
                self.theme.drag_float_text_color,
                self.theme.drag_float_background_color,
                self.theme.drag_float_border_color,
            ),
            (true, false) => (
                self.theme.drag_float_text_color_hovered,
                self.theme.drag_float_background_color_hovered,
                self.theme.drag_float_border_color_hovered,
            ),
            (_, true) => (
                self.theme.drag_float_text_color_active,
                self.theme.drag_float_background_color_active,
                self.theme.drag_float_border_color_active,
            ),
        };

        inner_ctrl_z.set_draw_self(true);
        inner_ctrl_z.set_draw_self_border_color(border_color);
        inner_ctrl_z.set_draw_self_background_color(background_color);

        s.clear();
        let _ = write!(
            s,
            "{:.1$}",
            self.value[2],
            usize::from(self.display_precision)
        );
        inner_ctrl_z.draw_text(
            true,
            None,
            0.0,
            &s,
            Align::Center,
            Align::Center,
            Wrap::Word,
            text_color,
        );

        frame.pop_ctrl();
        frame.pop_ctrl();

        changed_x | changed_y | changed_z
    }
}

fn x(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[0], state[1], state[2], state[3]])
}

fn value(state: &CtrlState) -> f32 {
    f32::from_le_bytes([state[4], state[5], state[6], state[7]])
}

fn set_x(state: &mut CtrlState, x: f32) {
    let bytes = x.to_le_bytes();
    state[0] = bytes[0];
    state[1] = bytes[1];
    state[2] = bytes[2];
    state[3] = bytes[3];
}

fn set_value(state: &mut CtrlState, value: f32) {
    let bytes = value.to_le_bytes();
    state[4] = bytes[0];
    state[5] = bytes[1];
    state[6] = bytes[2];
    state[7] = bytes[3];
}
