// TODO(yan): Split theme into themes for each component, so that when the user
// wants to edit something in the theme, they don't have to copy the whole
// struct.

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

    pub text_input_border_color: u32,
    pub text_input_border_color_hovered: u32,
    pub text_input_border_color_active: u32,
    pub text_input_background_color: u32,
    pub text_input_background_color_hovered: u32,
    pub text_input_background_color_active: u32,
    pub text_input_text_color: u32,
    pub text_input_text_color_hovered: u32,
    pub text_input_text_color_active: u32,
    pub text_input_height: f32,
    pub text_input_margin: f32,
    pub text_input_border: f32,
    pub text_input_overlay_max_height: f32,

    pub float_slider_border_color: u32,
    pub float_slider_border_color_hovered: u32,
    pub float_slider_border_color_active: u32,
    pub float_slider_background_color: u32,
    pub float_slider_background_color_hovered: u32,
    pub float_slider_background_color_active: u32,
    pub float_slider_text_color: u32,
    pub float_slider_text_color_hovered: u32,
    pub float_slider_text_color_active: u32,
    pub float_slider_height: f32,
    pub float_slider_margin: f32,
    pub float_slider_border: f32,

    pub int_slider_border_color: u32,
    pub int_slider_border_color_hovered: u32,
    pub int_slider_border_color_active: u32,
    pub int_slider_background_color: u32,
    pub int_slider_background_color_hovered: u32,
    pub int_slider_background_color_active: u32,
    pub int_slider_text_color: u32,
    pub int_slider_text_color_hovered: u32,
    pub int_slider_text_color_active: u32,
    pub int_slider_height: f32,
    pub int_slider_margin: f32,
    pub int_slider_border: f32,

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
    pub dropdown_margin: f32,
    pub dropdown_border: f32,
    pub dropdown_overlay_max_height: f32,

    pub panel_border_color: u32,
    pub panel_background_color: u32,
    pub panel_margin: f32,
    pub panel_border: f32,
    pub panel_padding: f32,
    pub panel_header_text_color: u32,
    pub panel_header_background_color: u32,
    pub panel_header_height: f32,

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

const TRANSPARENT: u32 = 0xffffff00;

const WINDOW_BACKGROUND_COLOR: u32 = 0x080808fa;
const WINDOW_BORDER_COLOR: u32 = 0x202020ff;
const WINDOW_HEADER_BACKGROUND_COLOR: u32 = 0x202080fa;

const BORDER_COLOR: u32 = 0x202020ff;
const BORDER_COLOR_HOVERED: u32 = 0x303030ff;
const BORDER_COLOR_ACTIVE: u32 = 0x505050ff;

const BACKGROUND_COLOR: u32 = 0;
const BACKGROUND_COLOR_HOVERED: u32 = 0x101010fa;
const BACKGROUND_COLOR_ACTIVE: u32 = 0x151515fa;

const TEXT_COLOR: u32 = 0xd0d0d0ff;
const TEXT_COLOR_HEADER: u32 = 0xf0f0f0ff;

impl Theme {
    pub const DEFAULT: Self = Self {
        button_border_color: BORDER_COLOR,
        button_border_color_hovered: BORDER_COLOR_HOVERED,
        button_border_color_active: BORDER_COLOR_ACTIVE,
        button_background_color: BACKGROUND_COLOR,
        button_background_color_hovered: BACKGROUND_COLOR_HOVERED,
        button_background_color_active: BACKGROUND_COLOR_ACTIVE,
        button_text_color: TEXT_COLOR,
        button_text_color_hovered: TEXT_COLOR,
        button_text_color_active: TEXT_COLOR,
        button_height: 30.0,
        button_margin: 2.0,
        button_border: 1.0,

        image_button_border_color: BORDER_COLOR,
        image_button_border_color_hovered: BORDER_COLOR_HOVERED,
        image_button_border_color_active: BORDER_COLOR_ACTIVE,
        image_button_background_color: BACKGROUND_COLOR,
        image_button_background_color_hovered: BACKGROUND_COLOR_HOVERED,
        image_button_background_color_active: BACKGROUND_COLOR_ACTIVE,
        image_button_width: 30.0,
        image_button_height: 30.0,
        image_button_margin: 2.0,
        image_button_border: 1.0,

        checkbox_handle_color: 0xffffff50,
        checkbox_handle_color_hovered: 0xffffff70,
        checkbox_handle_color_active: 0xffffffa0,
        checkbox_text_color: TEXT_COLOR,
        checkbox_text_color_hovered: TEXT_COLOR,
        checkbox_text_color_active: TEXT_COLOR,
        checkbox_width: 250.0,
        checkbox_height: 30.0,
        checkbox_margin: 2.0,
        checkbox_border: 1.0,

        text_border_color: TRANSPARENT,
        text_background_color: TRANSPARENT,
        text_text_color: TEXT_COLOR,
        text_margin: 0.0,
        text_border: 0.0,
        text_padding: 10.0,

        text_tooltip_border_color: BORDER_COLOR,
        text_tooltip_background_color: WINDOW_BACKGROUND_COLOR,
        text_tooltip_text_color: TEXT_COLOR,
        text_tooltip_border: 1.0,
        text_tooltip_padding: 10.0,

        text_input_border_color: BORDER_COLOR,
        text_input_border_color_hovered: BORDER_COLOR_HOVERED,
        text_input_border_color_active: BORDER_COLOR_ACTIVE,
        text_input_background_color: BACKGROUND_COLOR,
        text_input_background_color_hovered: BACKGROUND_COLOR_HOVERED,
        text_input_background_color_active: BACKGROUND_COLOR_ACTIVE,
        text_input_text_color: TEXT_COLOR,
        text_input_text_color_hovered: TEXT_COLOR,
        text_input_text_color_active: TEXT_COLOR,
        text_input_height: 30.0,
        text_input_margin: 2.0,
        text_input_border: 1.0,
        text_input_overlay_max_height: 400.0,

        float_slider_border_color: BORDER_COLOR,
        float_slider_border_color_hovered: BORDER_COLOR_HOVERED,
        float_slider_border_color_active: BORDER_COLOR_ACTIVE,
        float_slider_background_color: TRANSPARENT,
        float_slider_background_color_hovered: TRANSPARENT,
        float_slider_background_color_active: TRANSPARENT,
        float_slider_text_color: TEXT_COLOR,
        float_slider_text_color_hovered: TEXT_COLOR,
        float_slider_text_color_active: TEXT_COLOR,
        float_slider_height: 30.0,
        float_slider_margin: 2.0,
        float_slider_border: 1.0,

        int_slider_border_color: BORDER_COLOR,
        int_slider_border_color_hovered: BORDER_COLOR_HOVERED,
        int_slider_border_color_active: BORDER_COLOR_ACTIVE,
        int_slider_background_color: TRANSPARENT,
        int_slider_background_color_hovered: TRANSPARENT,
        int_slider_background_color_active: TRANSPARENT,
        int_slider_text_color: TEXT_COLOR,
        int_slider_text_color_hovered: TEXT_COLOR,
        int_slider_text_color_active: TEXT_COLOR,
        int_slider_height: 30.0,
        int_slider_margin: 2.0,
        int_slider_border: 1.0,

        dropdown_border_color: BORDER_COLOR,
        dropdown_border_color_hovered: BORDER_COLOR_HOVERED,
        dropdown_border_color_active: BORDER_COLOR_ACTIVE,
        dropdown_background_color: BACKGROUND_COLOR,
        dropdown_background_color_hovered: BACKGROUND_COLOR_HOVERED,
        dropdown_background_color_active: BACKGROUND_COLOR_ACTIVE,
        dropdown_text_color: TEXT_COLOR,
        dropdown_text_color_hovered: TEXT_COLOR,
        dropdown_text_color_active: TEXT_COLOR,
        dropdown_height: 30.0,
        dropdown_margin: 2.0,
        dropdown_border: 1.0,
        dropdown_overlay_max_height: 400.0,

        panel_border_color: TRANSPARENT,
        panel_background_color: TRANSPARENT,
        panel_margin: 5.0,
        panel_border: 0.0,
        panel_padding: 5.0,
        panel_header_text_color: TEXT_COLOR_HEADER,
        panel_header_background_color: WINDOW_HEADER_BACKGROUND_COLOR,
        panel_header_height: 20.0,

        window_border_color: BORDER_COLOR,
        window_border_color_hovered: WINDOW_BORDER_COLOR,
        window_background_color: WINDOW_BACKGROUND_COLOR,
        window_background_color_hovered: WINDOW_BACKGROUND_COLOR,
        window_border: 1.0,
        window_padding: 5.0,

        separator_color: BORDER_COLOR,
        separator_height: 1.0,
        separator_margin: 8.0,
    };
}
