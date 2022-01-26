use std::alloc::Allocator;
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
}

pub const GRAPH_LEN: usize = 60;

pub struct State {
    pub button_click_count: u64,
    pub input_text_submit_count: u64,
    pub input_text_cancel_count: u64,
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
    pub input_text_text_heap: String,
    pub input_text_text_inline: ArrayString<256>,
    pub drag_float_value: f32,
    pub drag_float_value_clamped: f32,
    pub drag_int_value: i32,
    pub drag_int_value_clamped: i32,
    pub dropdown_selected_option: Option<usize>,
}

pub fn draw_ui<A, TA>(frame: &mut guise::Frame<A, TA>, stats: &Stats, state: &mut State)
where
    A: Allocator + Clone,
    TA: Allocator,
{
    let time = stats.running_duration.as_secs_f32();
    let mut s: ArrayString<1024> = ArrayString::new();

    {
        guise::begin_window(frame, line!(), "26%", "1%", "73%", "98%");

        {
            guise::begin_panel_ex(frame, line!(), "100%", "50%", guise::Layout::Horizontal);

            {
                guise::begin_panel(frame, line!(), "50%", "100%");

                {
                    guise::begin_panel(frame, line!(), "100%", "15%");

                    guise::checkbox(
                        frame,
                        line!(),
                        &mut state.poll_platform_events,
                        "Poll Platform Events",
                    );

                    guise::end_panel(frame);
                }

                {
                    let mut panel_ctrl = guise::begin_panel(frame, line!(), "100%", "35%");

                    panel_ctrl.draw_text(
                        fmt!(
                            s,
                            "Button click count {}\nText Input submit count {}\nText Input cancel \
                             count {}",
                            state.button_click_count,
                            state.input_text_submit_count,
                            state.input_text_cancel_count,
                        ),
                        0xffffffff,
                    );

                    guise::end_panel(frame);
                }

                {
                    let mut panel_ctrl = guise::begin_panel(frame, line!(), "100%", "50%");

                    panel_ctrl.draw_text(
                        fmt!(
                            s,
                            "running time: {:.3}s\nframe count:  {}\nframe build time: \
                             {:.3}/{:.3}s (current/max)\nframe total time: {:.3}s\nframe ctrl \
                             count: {}",
                            time,
                            stats.frame_count,
                            stats.frame_build_duration.as_secs_f32(),
                            state.graph_frame_build_max,
                            stats.frame_total_duration.as_secs_f32(),
                            stats.frame_ctrl_count,
                        ),
                        0xffffffff,
                    );

                    guise::end_panel(frame);
                }

                guise::end_panel(frame);
            }

            {
                guise::begin_panel(frame, line!(), "50%", "100%");

                if guise::button(frame, 0, "A button with multiline text.\n And a footnote.") {
                    state.button_click_count += 1;
                }

                for i in 1..=50 {
                    if guise::button(frame, i, fmt!(s, "Button {}", i)) {
                        state.button_click_count += 1;
                    }
                }

                guise::end_panel(frame);
            }

            guise::end_panel(frame);
        }

        {
            guise::begin_panel_ex(frame, line!(), "100%", "50%", guise::Layout::Horizontal);

            {
                guise::begin_panel(frame, line!(), "50%", "100%");

                for i in 0..3 {
                    let i = i * 3;
                    let j = i + 1;
                    let k = i + 2;

                    guise::text(frame, i, TEXT);
                    guise::text_ex(frame, j, TEXT, guise::Align::Center);
                    guise::text_ex(frame, k, TEXT, guise::Align::End);
                }

                guise::end_panel(frame);
            }

            {
                let mut panel_ctrl = guise::begin_panel(frame, line!(), "50%", "100%");

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

                for i in 0..GRAPH_LEN {
                    let graph_frame_build_max = if state.graph_frame_build_max == 0.0 {
                        1.0
                    } else {
                        state.graph_frame_build_max
                    };
                    panel_ctrl.draw_rect(
                        false,
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 1.0 * height / 4.0,
                            0.23 * column_width,
                            0.23 * state.graph_frame_build[i] / graph_frame_build_max * height / 4.0,
                        ),
                        guise::Rect::ZERO,
                        if i == current_idx {
                            0xa4faa8ff
                        } else {
                            0xa4faa855
                        },
                        0,
                    );

                    let graph_command_count_max = if state.graph_command_count_max == 0 {
                        1.0
                    } else {
                        state.graph_command_count_max as f32
                    };
                    panel_ctrl.draw_rect(
                        false,
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
                        0,
                    );

                    let graph_vertex_count_max = if state.graph_vertex_count_max == 0 {
                        1.0
                    } else {
                        state.graph_vertex_count_max as f32
                    };
                    panel_ctrl.draw_rect(
                        false,
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
                        0,
                    );

                    let graph_index_count_max = if state.graph_index_count_max == 0 {
                        1.0
                    } else {
                        state.graph_index_count_max as f32
                    };
                    panel_ctrl.draw_rect(
                        false,
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
                        0,
                    );
                }

                guise::end_panel(frame);
            }

            guise::end_panel(frame);
        }

        guise::end_window(frame);
    }

    {
        let mut window_ctrl = guise::begin_window_ex(
            frame,
            line!(),
            "1%",
            "1%",
            "23%",
            "48%",
            guise::Layout::Free,
        );

        let inner_size = window_ctrl.inner_size();
        window_ctrl.draw_rect(
            false,
            guise::Rect::new(0.0, 0.0, inner_size.x, inner_size.y),
            guise::Rect::ONE,
            0xffffffff,
            0,
        );

        {
            let mut window_ctrl = guise::begin_window(frame, line!(), 5.0, 5.0, 100.0, 50.0);
            window_ctrl.draw_text_ex(
                false,
                guise::Vec2::ZERO,
                "Hello",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }

        {
            let mut window_ctrl = guise::begin_window(frame, line!(), 100.0, 100.0, 150.0, 50.0);
            window_ctrl.draw_text_ex(
                false,
                guise::Vec2::ZERO,
                "Traveller",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }

        {
            let mut window_ctrl = guise::begin_window(frame, line!(), 50.0, 250.0, 200.0, 100.0);
            window_ctrl.draw_text_ex(
                false,
                guise::Vec2::ZERO,
                "「こんにちは 世界」",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }

        guise::end_window(frame);
    }

    {
        guise::begin_window(frame, line!(), "1%", "51%", "23%", "48%");

        guise::dropdown(
            frame,
            line!(),
            "Damage Type",
            &[
                "Slashing",
                "Piercing",
                "Bludgeoning",
                "Fire",
                "Lightning",
                "Shadow",
                "Emotional",
            ],
            &mut state.dropdown_selected_option,
        );

        match guise::input_text(frame, line!(), &mut state.input_text_text_heap) {
            (_, guise::InputTextSubmit::None) => (),
            (_, guise::InputTextSubmit::Submit) => state.input_text_submit_count += 1,
            (_, guise::InputTextSubmit::Cancel) => state.input_text_cancel_count += 1,
        }

        match guise::input_text(frame, line!(), &mut state.input_text_text_inline) {
            (_, guise::InputTextSubmit::None) => (),
            (_, guise::InputTextSubmit::Submit) => state.input_text_submit_count += 1,
            (_, guise::InputTextSubmit::Cancel) => state.input_text_cancel_count += 1,
        }

        if guise::button(frame, line!(), "Clear") {
            state.input_text_text_heap.clear();
            state.input_text_text_inline.clear();
        }

        guise::drag_float(frame, line!(), &mut state.drag_float_value, 0.12);
        guise::drag_float_ex(
            frame,
            line!(),
            &mut state.drag_float_value_clamped,
            0.001,
            0.0,
            1.0,
        );

        guise::drag_int(frame, line!(), &mut state.drag_int_value, 0.12);
        guise::drag_int_ex(
            frame,
            line!(),
            &mut state.drag_int_value_clamped,
            0.1,
            0,
            100,
        );

        guise::end_window(frame);
    }
}
