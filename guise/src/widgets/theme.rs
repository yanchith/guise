// TODO(yan): Values for margin, border and padding could be split into
// horizontal and vertical, or even per rect side, but only do that if it is
// actually useful as it otherwise takes a lot of space in the Ctrl struct.

pub struct Theme {
    pub button_border_color: u32,
    pub button_border_color_hovered: u32,
    pub button_border_color_active: u32,
    pub button_background_color: u32,
    pub button_background_color_hovered: u32,
    pub button_background_color_active: u32,
    pub button_text_color: u32,
    pub button_text_color_hovered: u32,
    pub button_text_color_active: u32,
    pub button_height: f32,
    pub button_margin: f32,
    pub button_border: f32,

    pub image_button_border_color: u32,
    pub image_button_border_color_hovered: u32,
    pub image_button_border_color_active: u32,
    pub image_button_background_color: u32,
    pub image_button_background_color_hovered: u32,
    pub image_button_background_color_active: u32,
    pub image_button_width: f32,
    pub image_button_height: f32,
    pub image_button_margin: f32,
    pub image_button_border: f32,

    pub checkbox_handle_color: u32,
    pub checkbox_handle_color_hovered: u32,
    pub checkbox_handle_color_active: u32,
    pub checkbox_text_color: u32,
    pub checkbox_text_color_hovered: u32,
    pub checkbox_text_color_active: u32,
    pub checkbox_width: f32,
    pub checkbox_height: f32,
    pub checkbox_margin: f32,
    pub checkbox_border: f32,

    pub text_border_color: u32,
    pub text_background_color: u32,
    pub text_text_color: u32,
    pub text_margin: f32,
    pub text_border: f32,
    pub text_padding: f32,

    pub text_tooltip_border_color: u32,
    pub text_tooltip_background_color: u32,
    pub text_tooltip_text_color: u32,
    pub text_tooltip_border: f32,
    pub text_tooltip_padding: f32,

    pub input_text_border_color: u32,
    pub input_text_border_color_hovered: u32,
    pub input_text_border_color_active: u32,
    pub input_text_background_color: u32,
    pub input_text_background_color_hovered: u32,
    pub input_text_background_color_active: u32,
    pub input_text_text_color: u32,
    pub input_text_text_color_hovered: u32,
    pub input_text_text_color_active: u32,
    pub input_text_height: f32,
    pub input_text_margin: f32,
    pub input_text_border: f32,

    pub drag_float_border_color: u32,
    pub drag_float_border_color_hovered: u32,
    pub drag_float_border_color_active: u32,
    pub drag_float_background_color: u32,
    pub drag_float_background_color_hovered: u32,
    pub drag_float_background_color_active: u32,
    pub drag_float_text_color: u32,
    pub drag_float_text_color_hovered: u32,
    pub drag_float_text_color_active: u32,
    pub drag_float_height: f32,
    pub drag_float_margin: f32,
    pub drag_float_border: f32,

    pub drag_int_border_color: u32,
    pub drag_int_border_color_hovered: u32,
    pub drag_int_border_color_active: u32,
    pub drag_int_background_color: u32,
    pub drag_int_background_color_hovered: u32,
    pub drag_int_background_color_active: u32,
    pub drag_int_text_color: u32,
    pub drag_int_text_color_hovered: u32,
    pub drag_int_text_color_active: u32,
    pub drag_int_height: f32,
    pub drag_int_margin: f32,
    pub drag_int_border: f32,

    pub dropdown_border_color: u32,
    pub dropdown_border_color_hovered: u32,
    pub dropdown_border_color_active: u32,
    pub dropdown_background_color: u32,
    pub dropdown_background_color_hovered: u32,
    pub dropdown_background_color_active: u32,
    pub dropdown_text_color: u32,
    pub dropdown_text_color_hovered: u32,
    pub dropdown_text_color_active: u32,
    pub dropdown_height: f32,
    pub dropdown_overlay_max_height: f32,
    pub dropdown_margin: f32,
    pub dropdown_border: f32,

    pub panel_border_color: u32,
    pub panel_background_color: u32,
    pub panel_border: f32,
    pub panel_padding: f32,

    pub window_border_color: u32,
    pub window_border_color_hovered: u32,
    pub window_background_color: u32,
    pub window_background_color_hovered: u32,
    pub window_border: f32,
    pub window_padding: f32,

    pub separator_color: u32,
    pub separator_height: f32,
    pub separator_margin: f32,
}

impl Theme {
    pub const DEFAULT: Self = Self {
        button_border_color: 0xa0a0a0ff,
        button_border_color_hovered: 0xb0b0b0ff,
        button_border_color_active: 0xd0d0d0ff,
        button_background_color: 0xa0a0a000,
        button_background_color_hovered: 0xa0a0a070,
        button_background_color_active: 0xa0a0a0ff,
        button_text_color: 0xffffffff,
        button_text_color_hovered: 0xffffffff,
        button_text_color_active: 0xffffffff,
        button_height: 40.0,
        button_margin: 2.0,
        button_border: 1.0,

        image_button_border_color: 0xa0a0a0ff,
        image_button_border_color_hovered: 0xb0b0b0ff,
        image_button_border_color_active: 0xd0d0d0ff,
        image_button_background_color: 0xa0a0a000,
        image_button_background_color_hovered: 0xa0a0a070,
        image_button_background_color_active: 0xa0a0a0ff,
        image_button_width: 40.0,
        image_button_height: 40.0,
        image_button_margin: 2.0,
        image_button_border: 1.0,

        checkbox_handle_color: 0xffffff50,
        checkbox_handle_color_hovered: 0xffffff70,
        checkbox_handle_color_active: 0xffffffa0,
        checkbox_text_color: 0xffffffff,
        checkbox_text_color_hovered: 0xffffffff,
        checkbox_text_color_active: 0xffffffff,
        checkbox_width: 250.0,
        checkbox_height: 40.0,
        checkbox_margin: 2.0,
        checkbox_border: 1.0,

        text_border_color: 0xffffff00,
        text_background_color: 0xffffff00,
        text_text_color: 0xffffffff,
        text_margin: 0.0,
        text_border: 0.0,
        text_padding: 10.0,

        text_tooltip_border_color: 0xa0a0a0ff,
        text_tooltip_background_color: 0x606060d0,
        text_tooltip_text_color: 0xffffffff,
        text_tooltip_border: 1.0,
        text_tooltip_padding: 10.0,

        input_text_border_color: 0xa0a0a050,
        input_text_border_color_hovered: 0xa0a0a070,
        input_text_border_color_active: 0xa0a0a0ff,
        input_text_background_color: 0xffffff00,
        input_text_background_color_hovered: 0xffffff00,
        input_text_background_color_active: 0xffffff00,
        input_text_text_color: 0xffffffff,
        input_text_text_color_hovered: 0xffffffff,
        input_text_text_color_active: 0xffffffff,
        input_text_height: 40.0,
        input_text_margin: 2.0,
        input_text_border: 1.0,

        drag_float_border_color: 0xa0a0a050,
        drag_float_border_color_hovered: 0xa0a0a070,
        drag_float_border_color_active: 0xa0a0a0ff,
        drag_float_background_color: 0xffffff00,
        drag_float_background_color_hovered: 0xffffff00,
        drag_float_background_color_active: 0xffffff00,
        drag_float_text_color: 0xffffffff,
        drag_float_text_color_hovered: 0xffffffff,
        drag_float_text_color_active: 0xffffffff,
        drag_float_height: 40.0,
        drag_float_margin: 2.0,
        drag_float_border: 1.0,

        drag_int_border_color: 0xa0a0a050,
        drag_int_border_color_hovered: 0xa0a0a070,
        drag_int_border_color_active: 0xa0a0a0ff,
        drag_int_background_color: 0xffffff00,
        drag_int_background_color_hovered: 0xffffff00,
        drag_int_background_color_active: 0xffffff00,
        drag_int_text_color: 0xffffffff,
        drag_int_text_color_hovered: 0xffffffff,
        drag_int_text_color_active: 0xffffffff,
        drag_int_height: 40.0,
        drag_int_margin: 2.0,
        drag_int_border: 1.0,

        dropdown_border_color: 0xa0a0a0ff,
        dropdown_border_color_hovered: 0xb0b0b0ff,
        dropdown_border_color_active: 0xd0d0d0ff,
        dropdown_background_color: 0xa0a0a000,
        dropdown_background_color_hovered: 0xa0a0a070,
        dropdown_background_color_active: 0xa0a0a0ff,
        dropdown_text_color: 0xffffffff,
        dropdown_text_color_hovered: 0xffffffff,
        dropdown_text_color_active: 0xffffffff,
        dropdown_height: 40.0,
        dropdown_overlay_max_height: 400.0,
        dropdown_margin: 2.0,
        dropdown_border: 1.0,

        panel_border_color: 0xa0a0a050,
        panel_background_color: 0xffffff00,
        panel_border: 1.0,
        panel_padding: 2.0,

        window_border_color: 0xa0a0a0ff,
        window_border_color_hovered: 0xb0b0b0ff,
        window_background_color: 0x606060d0,
        window_background_color_hovered: 0x626262d0,
        window_border: 1.0,
        window_padding: 5.0,

        separator_color: 0xffffff50,
        separator_height: 1.0,
        separator_margin: 8.0,
    };
}
