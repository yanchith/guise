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
    pub input_text_text_heap: guise::AsciiVec<Global>,
    pub input_text_text_inline: guise::AsciiArrayVec<64>,
    pub drag_float_value: f32,
    pub drag_float_value_clamped: f32,
    pub drag_int_value: i32,
    pub drag_int_value_clamped: i32,
    pub dropdown_selected_option: Option<usize>,
}

pub fn draw_ui<A: Allocator + Clone>(
    frame: &mut guise::Frame<A>,
    stats: &Stats,
    state: &mut State,
) {
    let time = stats.running_duration.as_secs_f32();
    let mut s: ArrayString<1024> = ArrayString::new();

    {
        guise::begin_window(frame, line!(), "26%", "1%", "73%", "98%");

        {
            guise::Panel::new(line!(), "100%", "50%")
                .set_layout(guise::Layout::Horizontal)
                .begin(frame);

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
                    guise::begin_panel(frame, line!(), "100%", "35%");
                    guise::text(
                        frame,
                        0,
                        fmt!(
                            s,
                            "Button click count {}\nText Input submit count {}\nText Input cancel \
                             count {}",
                            state.button_click_count,
                            state.input_text_submit_count,
                            state.input_text_cancel_count,
                        ),
                    );
                    guise::end_panel(frame);
                }

                {
                    guise::begin_panel(frame, line!(), "100%", "50%");
                    guise::text(
                        frame,
                        0,
                        fmt!(
                            s,
                            "running time: {:.3}s\nframe count:  {}\nframe build time: \
                             {:.3}/{:.3}s (current/max)\nframe total time: {:.3}s\nframe ctrl \
                             count: {}\nwant capture keyboard {}\nwant capture mouse {}",
                            time,
                            stats.frame_count,
                            stats.frame_build_duration.as_secs_f32(),
                            state.graph_frame_build_max,
                            stats.frame_total_duration.as_secs_f32(),
                            stats.frame_ctrl_count,
                            stats.want_capture_keyboard,
                            stats.want_capture_mouse,
                        ),
                    );
                    guise::end_panel(frame);
                }

                guise::end_panel(frame);
            }

            {
                guise::begin_panel(frame, line!(), "50%", "100%");

                if guise::Button::new(0, "<image>")
                    .set_image(0)
                    .set_tooltip("An image button")
                    .show(frame)
                {
                    state.button_click_count += 1;
                }

                if guise::button(frame, 1, "A button with multiline text.\n And a footnote.") {
                    state.button_click_count += 1;
                }

                for i in 2..=50 {
                    if guise::button(frame, i, fmt!(s, "Button {}", i)) {
                        state.button_click_count += 1;
                    }
                }

                guise::end_panel(frame);
            }

            guise::end_panel(frame);
        }

        {
            guise::Panel::new(line!(), "100%", "50%")
                .set_layout(guise::Layout::Horizontal)
                .begin(frame);

            {
                guise::begin_panel(frame, line!(), "50%", "100%");

                for i in 0..3 {
                    let i = i * 3;
                    let j = i + 1;
                    let k = i + 2;

                    guise::text(frame, i, TEXT);
                    guise::Text::new(j, TEXT)
                        .set_horizontal_align(guise::Align::Center)
                        .show(frame);
                    guise::Text::new(k, TEXT)
                        .set_horizontal_align(guise::Align::End)
                        .show(frame);
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
                            0.23 * state.graph_frame_build[i] / graph_frame_build_max * height
                                / 4.0,
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
        let mut window_ctrl = guise::Window::new(line!(), "1%", "1%", "23%", "48%")
            .set_layout(guise::Layout::Free)
            .begin(frame);

        let inner_size = window_ctrl.inner_size();
        window_ctrl.draw_rect(
            false,
            guise::Rect::new(0.0, 0.0, inner_size.x, inner_size.y),
            guise::Rect::ONE,
            0xffffffff,
            0,
        );

        {
            let mut window_ctrl = guise::Window::new(line!(), 5.0, 5.0, 150.0, 50.0)
                .set_resizable(false)
                .begin(frame);
            window_ctrl.draw_text(
                false,
                None,
                0.0,
                "This window not resizable",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }

        {
            let mut window_ctrl = guise::Window::new(line!(), 100.0, 100.0, 150.0, 50.0)
                .set_movable(false)
                .begin(frame);
            window_ctrl.draw_text(
                false,
                None,
                0.0,
                "This window is not movable",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }

        {
            let mut window_ctrl = guise::Window::new(line!(), "5%", "80%", "90%", "15%")
                .set_movable(false)
                .set_resizable(false)
                .begin(frame);
            window_ctrl.draw_text(
                false,
                None,
                0.0,
                "This window is neither movable nor resizable",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }

        {
            let mut window_ctrl = guise::begin_window(frame, line!(), 20.0, 160.0, 200.0, 60.0);
            window_ctrl.draw_text(
                false,
                None,
                0.0,
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

        guise::drag_float(frame, line!(), &mut state.drag_float_value);
        guise::DragFloat::new(line!(), &mut state.drag_float_value_clamped)
            .set_speed(0.00001)
            .set_min(0.0)
            .set_max(1.0)
            .set_display_precision(8)
            .show(frame);

        guise::drag_int(frame, line!(), &mut state.drag_int_value, "Fast Int (unclamped)");
        guise::DragInt::new(line!(), &mut state.drag_int_value_clamped, "Slow Int (clamped)")
            .set_speed(0.05)
            .set_min(0)
            .set_max(100)
            .show(frame);

        guise::end_window(frame);
    }
}
