use std::alloc::{Allocator, Global};
use std::fmt::Write as _;
use std::time::Duration;

use arrayvec::ArrayString;

macro_rules! fmt {
    ($dst:expr, $($fmt:tt)*) => ({
        $dst.clear();
        std::write!($dst, $($fmt)*).unwrap();

        &$dst
    });
}

static TEXT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
                     tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, \
                     quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo \
                     consequat.";

pub struct Stats {
    pub running_duration: Duration,
    pub frame_count: u64,
    pub frame_build_duration: Duration,
    pub frame_total_duration: Duration,
    pub frame_draw_list_command_count: usize,
    pub frame_draw_list_vertex_count: usize,
    pub frame_draw_list_index_count: usize,
    pub frame_ctrl_count: usize,
    pub want_capture_keyboard: bool,
    pub want_capture_mouse: bool,
}

pub const GRAPH_LEN: usize = 60;

pub struct State {
    pub button_click_count: u64,
    pub text_input_submit_count: u64,
    pub text_input_cancel_count: u64,
    pub poll_platform_events: bool,
    pub graph: [f32; GRAPH_LEN],
    pub graph_max: f32,
    pub graph_frame_build: [f32; GRAPH_LEN],
    pub graph_frame_build_max: f32,
    pub graph_command_count: [usize; GRAPH_LEN],
    pub graph_command_count_max: usize,
    pub graph_vertex_count: [usize; GRAPH_LEN],
    pub graph_vertex_count_max: usize,
    pub graph_index_count: [usize; GRAPH_LEN],
    pub graph_index_count_max: usize,
    pub text_input_heap: guise::VecString<Global>,
    pub text_input_inline: ArrayString<64>,
    pub float_slider_value: f32,
    pub float_slider_value_clamped: f32,
    pub float_slider2_value: [f32; 2],
    pub float_slider3_value: [f32; 3],
    pub float_slider4_value: [f32; 4],
    pub int_slider_value: i32,
    pub int_slider_value_clamped: i32,
    pub int_slider2_value: [i32; 2],
    pub int_slider3_value: [i32; 3],
    pub int_slider4_value: [i32; 4],
    pub dropdown1_selected_option: Option<usize>,
    pub dropdown2_selected_option: Option<usize>,
}

pub fn draw_ui<A: Allocator + Clone>(
    frame: &mut guise::Frame<A>,
    stats: &Stats,
    state: &mut State,
) {
    let texture_id = frame.font_atlas_texture_id();
    let time = stats.running_duration.as_secs_f32();
    let mut s: ArrayString<1024> = ArrayString::new();

    if let Some((window, _)) = guise::begin_window(frame, line!(), "41%", "1%", "58%", "98%") {
        guise::Panel::new(line!(), "100%", "48%", "Panel header text")
            .set_layout(guise::Layout::Horizontal)
            .set_draw_padding(false)
            .begin(frame);

        {
            guise::Panel::new(line!(), "50%", "100%", "System")
                .set_draw_border(false)
                .begin(frame);

            guise::checkbox(
                frame,
                line!(),
                &mut state.poll_platform_events,
                "Poll Platform Events",
            );

            guise::separator(frame, line!());

            guise::text(
                frame,
                line!(),
                fmt!(
                    s,
                    "Button click count {}\nText Input submit count {}\nText Input cancel count {}",
                    state.button_click_count,
                    state.text_input_submit_count,
                    state.text_input_cancel_count,
                ),
            );

            guise::separator(frame, line!());

            guise::text_with_align(
                frame,
                line!(),
                fmt!(
                    s,
                    "running time: {:.3}s\nframe count:  {}\nframe build time: {:.3}/{:.3}s \
                     (current/max)\nframe total time: {:.3}s\nframe ctrl count: {}\nwant capture \
                     keyboard {}\nwant capture mouse {}",
                    time,
                    stats.frame_count,
                    stats.frame_build_duration.as_secs_f32(),
                    state.graph_frame_build_max,
                    stats.frame_total_duration.as_secs_f32(),
                    stats.frame_ctrl_count,
                    stats.want_capture_keyboard,
                    stats.want_capture_mouse,
                ),
                guise::Align::Start,
            );

            guise::end_panel(frame);
        }

        {
            guise::Panel::new(
                line!(),
                "50%",
                "100%",
                "A few buttons for your consideration",
            )
            .set_resize_height_to_fit_content(true)
            .set_draw_border(false)
            .begin(frame);

            if guise::image_button_with_tooltip(frame, line!(), 0, "An image button") {
                state.button_click_count += 1;
            }

            if guise::button_with_tooltip(frame, line!(), "A button with tooltip", TEXT) {
                state.button_click_count += 1;
            }

            for i in 0..=10 {
                frame.push_id_namespace(i);
                if guise::button(frame, line!(), fmt!(s, "Button {}", i)) {
                    state.button_click_count += 1;
                }
                frame.pop_id_namespace();
            }

            guise::end_panel(frame);
        }

        guise::end_panel(frame);

        guise::separator(frame, line!());

        {
            guise::Panel::new(line!(), "100%", "48%", "Another panel header text")
                .set_layout(guise::Layout::Horizontal)
                .set_draw_padding(false)
                .set_draw_header(false)
                .begin(frame);

            {
                guise::Panel::new(line!(), "50%", "100%", "Drawing Text")
                    .set_draw_border(false)
                    .begin(frame);

                for i in 0..3 {
                    let i = i * 3;
                    let j = i + 1;
                    let k = i + 2;

                    guise::text_with_align(frame, i, TEXT, guise::Align::Start);
                    guise::text_with_align(frame, j, TEXT, guise::Align::Center);
                    guise::text_with_align(frame, k, TEXT, guise::Align::End);
                }

                guise::end_panel(frame);
            }

            {
                let mut panel_ctrl = guise::Panel::new(line!(), "50%", "100%", "Drawing Graphs")
                    .set_draw_border(false)
                    .set_draw_header(true)
                    .begin(frame);

                let size = panel_ctrl.inner_size();
                let width = size.x;
                let height = size.y;
                let column_width = width / GRAPH_LEN as f32;

                let current_idx = stats.frame_count as usize % GRAPH_LEN;
                let current_frame_build_duration = stats.frame_build_duration.as_secs_f32();
                let current_draw_list_command_count = stats.frame_draw_list_command_count;
                let current_draw_list_vertex_count = stats.frame_draw_list_vertex_count;
                let current_draw_list_index_count = stats.frame_draw_list_index_count;

                state.graph_frame_build[current_idx] = current_frame_build_duration;
                state.graph_command_count[current_idx] = current_draw_list_command_count;
                state.graph_vertex_count[current_idx] = current_draw_list_vertex_count;
                state.graph_index_count[current_idx] = current_draw_list_index_count;

                if current_frame_build_duration > state.graph_frame_build_max {
                    state.graph_frame_build_max = current_frame_build_duration;
                }
                if current_draw_list_command_count > state.graph_command_count_max {
                    state.graph_command_count_max = current_draw_list_command_count;
                }
                if current_draw_list_vertex_count > state.graph_vertex_count_max {
                    state.graph_vertex_count_max = current_draw_list_vertex_count;
                }
                if current_draw_list_index_count > state.graph_index_count_max {
                    state.graph_index_count_max = current_draw_list_index_count;
                }

                // TODO(yan): @Bug The draw_rect calls for the graphs seem to
                // ignore margin/padding/whatever.
                for i in 0..GRAPH_LEN {
                    let graph_frame_build_max = if state.graph_frame_build_max == 0.0 {
                        1.0
                    } else {
                        state.graph_frame_build_max
                    };

                    panel_ctrl.draw_rect(
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 1.0 * height / 4.0,
                            0.23 * column_width,
                            0.23 * state.graph_frame_build[i] / graph_frame_build_max * height
                                / 4.0,
                        ),
                        guise::Rect::ZERO,
                        if i == current_idx {
                            0xa4faa8ff
                        } else {
                            0xa4faa855
                        },
                        texture_id,
                    );

                    let graph_command_count_max = if state.graph_command_count_max == 0 {
                        1.0
                    } else {
                        state.graph_command_count_max as f32
                    };
                    panel_ctrl.draw_rect(
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 2.0 * height / 4.0,
                            0.23 * column_width,
                            0.23 * state.graph_command_count[i] as f32 / graph_command_count_max
                                * height
                                / 4.0,
                        ),
                        guise::Rect::ZERO,
                        if i == current_idx {
                            0xfbd160ff
                        } else {
                            0xfbd16055
                        },
                        texture_id,
                    );

                    let graph_vertex_count_max = if state.graph_vertex_count_max == 0 {
                        1.0
                    } else {
                        state.graph_vertex_count_max as f32
                    };
                    panel_ctrl.draw_rect(
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 3.0 * height / 4.0,
                            0.23 * column_width,
                            0.23 * state.graph_vertex_count[i] as f32 / graph_vertex_count_max
                                * height
                                / 4.0,
                        ),
                        guise::Rect::ZERO,
                        if i == current_idx {
                            0x29a0b1ff
                        } else {
                            0x29a0b155
                        },
                        texture_id,
                    );

                    let graph_index_count_max = if state.graph_index_count_max == 0 {
                        1.0
                    } else {
                        state.graph_index_count_max as f32
                    };
                    panel_ctrl.draw_rect(
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 4.0 * height / 4.0,
                            0.23 * column_width,
                            0.23 * state.graph_index_count[i] as f32 / graph_index_count_max
                                * height
                                / 4.0,
                        ),
                        guise::Rect::ZERO,
                        if i == current_idx {
                            0xf95011ff
                        } else {
                            0xf9501155
                        },
                        texture_id,
                    );
                }

                guise::end_panel(frame);
            }

            guise::end_panel(frame);
        }

        window.end(frame);
    }

    if let Some((window, mut window_ctrl)) = guise::begin_window_with_layout(
        frame,
        line!(),
        "1%",
        "1%",
        "39%",
        "48%",
        guise::Layout::Free,
    ) {
        let inner_size = window_ctrl.inner_size();
        window_ctrl.draw_rect(
            guise::Rect::new(0.0, 0.0, inner_size.x, inner_size.y),
            guise::Rect::ONE,
            0xffffffff,
            texture_id,
        );

        if let Some((window, mut window_ctrl)) = guise::begin_window_with_layout_options(
            frame,
            line!(),
            5.0,
            5.0,
            150.0,
            50.0,
            guise::Layout::Vertical,
            &guise::WindowOptions {
                resizable: false,
                ..guise::WindowOptions::default()
            },
        ) {
            window_ctrl.draw_text(
                "This window not resizable",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x907030ff,
            );

            window.end(frame);
        }

        if let Some((window, mut window_ctrl)) = guise::begin_window_with_layout_options(
            frame,
            line!(),
            100.0,
            100.0,
            150.0,
            50.0,
            guise::Layout::Vertical,
            &guise::WindowOptions {
                movable: false,
                ..guise::WindowOptions::default()
            },
        ) {
            window_ctrl.draw_text(
                "This window is not movable",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x907030ff,
            );

            window.end(frame);
        }

        if let Some((window, mut window_ctrl)) = guise::begin_window_with_layout_options(
            frame,
            line!(),
            10.0,
            "80%",
            -20.0,
            "15%",
            guise::Layout::Vertical,
            &guise::WindowOptions {
                movable: false,
                resizable: false,
                ..guise::WindowOptions::default()
            },
        ) {
            window_ctrl.draw_text(
                "This window is neither movable nor resizable",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x907030ff,
            );
            window.end(frame);
        }

        if let Some((window, mut window_ctrl)) =
            guise::begin_window(frame, line!(), 20.0, 160.0, 200.0, 60.0)
        {
            window_ctrl.draw_text(
                "「こんにちは 世界」",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x907030ff,
            );

            window.end(frame);
        }

        window.end(frame);
    }

    if let Some((window, _)) = guise::begin_window(frame, line!(), "1%", "51%", "39%", "48%") {
        guise::text(frame, line!(), "Dropdowns");

        static DAMAGE_TYPES: &[&str] = &[
            "Slashing",
            "Piercing",
            "Bludgeoning",
            "Fire",
            "Lightning",
            "Shadow",
            "Emotional",
        ];

        guise::dropdown(
            frame,
            line!(),
            "Damage Type",
            DAMAGE_TYPES,
            &mut state.dropdown1_selected_option,
        );
        guise::Dropdown::new(
            line!(),
            "Damage Type (allows unselect)",
            DAMAGE_TYPES,
            &mut state.dropdown2_selected_option,
        )
        .set_allow_unselect(true)
        .show(frame);

        guise::separator(frame, line!());
        guise::text(frame, line!(), "Text inputs");

        guise::text_input_with_callback(
            frame,
            line!(),
            &mut state.text_input_inline,
            "Inline String",
            |data, _| match data.action {
                guise::TextInputAction::None => (),
                guise::TextInputAction::Submit => state.text_input_submit_count += 1,
                guise::TextInputAction::Cancel => state.text_input_cancel_count += 1,
            },
        );

        guise::text_input_with_callback_autocomplete(
            frame,
            line!(),
            &mut state.text_input_heap,
            "Heap String (with autocomplete)",
            |data, _| match data.action {
                guise::TextInputAction::None => (),
                guise::TextInputAction::Submit => state.text_input_submit_count += 1,
                guise::TextInputAction::Cancel => state.text_input_cancel_count += 1,
            },
            &[
                "Mag Iontach (The Brilliant Plain)",
                "Réimse an Nádúr (Realm of Nature)",
                "Tír an t-Ór (Land of Gold)",
                "Tír Naomhtha (The Hallowed Land)",
                "Domhan an Filleadh (World of the Return)",
                "Réimse na Seaimpíní (Realm of Champions)",
                "Tír Cheart (The Righteous Land)",
                "Tír Ardaithe (The Exalted Land)",
                "Tír Geal (The Bright Land)",
                "Tír an Dath (Land of Color)",
            ],
        );

        if guise::button(frame, line!(), "Clear") {
            state.text_input_heap.clear();
            state.text_input_inline.clear();
        }

        guise::separator(frame, line!());
        guise::text(frame, line!(), "Sliders");

        guise::float_slider(
            frame,
            line!(),
            &mut state.float_slider_value,
            "Fast Float (unclamped)",
        );

        guise::float_slider_with_speed_min_max_precision(
            frame,
            line!(),
            &mut state.float_slider_value_clamped,
            "Slow Float (clamped)",
            0.00001,
            0.0,
            0.1,
            6,
        );

        guise::float2_slider(frame, line!(), &mut state.float_slider2_value, "Vec2");
        guise::float3_slider(frame, line!(), &mut state.float_slider3_value, "Vec3");
        guise::float4_slider(frame, line!(), &mut state.float_slider4_value, "Vec4");

        guise::int_slider(
            frame,
            line!(),
            &mut state.int_slider_value,
            "Fast Int (unclamped)",
        );

        guise::int_slider_with_speed_min_max(
            frame,
            line!(),
            &mut state.int_slider_value_clamped,
            "Slow Int (clamped)",
            0.05,
            0,
            100,
        );

        guise::int2_slider(frame, line!(), &mut state.int_slider2_value, "IVec2");
        guise::int3_slider(frame, line!(), &mut state.int_slider3_value, "IVec3");
        guise::int4_slider(frame, line!(), &mut state.int_slider4_value, "IVec4");

        window.end(frame);
    }

    if let Some((window, _)) = guise::begin_window(frame, line!(), "1%", "1%", 350.0, 300.0) {
        guise::Panel::new(line!(), "100%", "100%", "RESIZE_TO_FIT test")
            .set_resize_height_to_fit_content(true)
            .begin(frame);

        // TODO(yan): The "Can you see me?" inside has inconsistent
        // padding/margin if the panel starts with 0 height and grows because of
        // the RESIZE_TO_FIT_VERTICAL flag. It works properly if the height is
        // large enough to contain everything from the start and then shrinks
        // becuase of RESIZE_TO_FIT_VERTICAL.
        guise::button(frame, line!(), "Hello");
        guise::text(frame, line!(), "Can you see me?");
        guise::button(frame, line!(), "Bye");

        guise::end_panel(frame);

        window.end(frame);
    }
}
