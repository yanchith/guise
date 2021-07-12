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
    pub frame_ctrl_count: usize,
}

pub const GRAPH_LEN: usize = 60;

pub struct State {
    pub button_click_count: u64,
    pub graph: [f32; GRAPH_LEN],
    pub graph_max: f32,
    pub graph_frame_build: [f32; GRAPH_LEN],
    pub graph_frame_build_max: f32,
    pub graph_command_count: [usize; GRAPH_LEN],
    pub graph_command_count_max: usize,
    pub graph_vertex_count: [usize; GRAPH_LEN],
    pub graph_vertex_count_max: usize,
    pub text_input_text_heap: String,
    pub text_input_text_inline: ArrayString<256>,
}

pub fn draw_ui(frame: &mut guise::Frame, stats: &Stats, state: &mut State) {
    let time = stats.running_duration.as_secs_f32();
    let theme = &guise::Theme::DEFAULT;
    let mut s: ArrayString<1024> = ArrayString::new();

    if guise::begin_window(
        frame,
        line!(),
        theme,
        guise::Layout::Vertical,
        "26%",
        "1%",
        "73%",
        "98%",
    )
    .is_some()
    {
        if guise::begin_panel(
            frame,
            line!(),
            theme,
            guise::Layout::Horizontal,
            "100%",
            "50%",
        )
        .is_some()
        {
            if guise::begin_panel(
                frame,
                line!(),
                theme,
                guise::Layout::Vertical,
                "50%",
                "100%",
            )
            .is_some()
            {
                if let Some(mut ctrl) = guise::begin_panel(
                    frame,
                    line!(),
                    theme,
                    guise::Layout::Vertical,
                    "100%",
                    "50%",
                ) {
                    ctrl.draw_text(
                        true,
                        fmt!(s, "Button click count {}", state.button_click_count),
                        guise::Align::Start,
                        guise::Align::Center,
                        guise::Wrap::Word,
                        0xffffffff,
                    );

                    guise::end_panel(frame);
                }

                if let Some(mut ctrl) = guise::begin_panel(
                    frame,
                    line!(),
                    theme,
                    guise::Layout::Vertical,
                    "100%",
                    "50%",
                ) {
                    ctrl.draw_text(
                        true,
                        fmt!(
                            s,
                            "running time: {:.3}s\nframe count:  {}\nframe build time: {:.3}/{:.3}s (current/max)\nframe total time: {:.3}s\nframe ctrl count: {}",
                            time,
                            stats.frame_count,
                            stats.frame_build_duration.as_secs_f32(),
                            state.graph_frame_build_max,
                            stats.frame_total_duration.as_secs_f32(),
                            stats.frame_ctrl_count,
                        ),
                        guise::Align::Start,
                        guise::Align::Center,
                        guise::Wrap::Word,
                        0xffffffff,
                    );

                    guise::end_panel(frame);
                }

                guise::end_panel(frame);
            }

            if guise::begin_panel(
                frame,
                line!(),
                theme,
                guise::Layout::Vertical,
                "50%",
                "100%",
            )
            .is_some()
            {
                if guise::button(
                    frame,
                    0,
                    theme,
                    "A button with multiline text.\n And a footnote.",
                ) {
                    state.button_click_count += 1;
                }

                for i in 1..=50 {
                    if guise::button(frame, i, theme, fmt!(s, "Button {}", i)) {
                        state.button_click_count += 1;
                    }
                }

                guise::end_panel(frame);
            }

            guise::end_panel(frame);
        }

        if guise::begin_panel(
            frame,
            line!(),
            theme,
            guise::Layout::Horizontal,
            "100%",
            "50%",
        )
        .is_some()
        {
            if guise::begin_panel(
                frame,
                line!(),
                theme,
                guise::Layout::Vertical,
                "50%",
                "100%",
            )
            .is_some()
            {
                for i in 1..=3 {
                    let i = i * 3;
                    let j = i + 1;
                    let k = i + 2;

                    if let Some(mut ctrl) =
                        guise::begin_panel(frame, i, theme, guise::Layout::Vertical, "100%", 100.0)
                    {
                        ctrl.draw_text(
                            true,
                            TEXT,
                            guise::Align::Start,
                            guise::Align::Center,
                            guise::Wrap::Word,
                            0xffffffff,
                        );
                        guise::end_panel(frame);
                    }

                    if let Some(mut ctrl) =
                        guise::begin_panel(frame, j, theme, guise::Layout::Vertical, "100%", 100.0)
                    {
                        ctrl.draw_text(
                            true,
                            TEXT,
                            guise::Align::Center,
                            guise::Align::Center,
                            guise::Wrap::Word,
                            0xffffffff,
                        );
                        guise::end_panel(frame);
                    }

                    if let Some(mut ctrl) =
                        guise::begin_panel(frame, k, theme, guise::Layout::Vertical, "100%", 100.0)
                    {
                        ctrl.draw_text(
                            true,
                            TEXT,
                            guise::Align::End,
                            guise::Align::Center,
                            guise::Wrap::Word,
                            0xffffffff,
                        );
                        guise::end_panel(frame);
                    }
                }

                guise::end_panel(frame);
            }

            if let Some(mut ctrl) = guise::begin_panel(
                frame,
                line!(),
                theme,
                guise::Layout::Vertical,
                "50%",
                "100%",
            ) {
                let extents = ctrl.inner_extents();
                let width = extents.x;
                let height = extents.y;
                let column_width = width / GRAPH_LEN as f32;

                let current_idx = stats.frame_count as usize % GRAPH_LEN;
                let current_frame_build_duration = stats.frame_build_duration.as_secs_f32();
                let current_draw_list_command_count = stats.frame_draw_list_command_count;
                let current_draw_list_vertex_count = stats.frame_draw_list_vertex_count;

                state.graph_frame_build[current_idx] = current_frame_build_duration;
                state.graph_command_count[current_idx] = current_draw_list_command_count;
                state.graph_vertex_count[current_idx] = current_draw_list_vertex_count;

                if current_frame_build_duration > state.graph_frame_build_max {
                    state.graph_frame_build_max = current_frame_build_duration;
                }
                if current_draw_list_command_count > state.graph_command_count_max {
                    state.graph_command_count_max = current_draw_list_command_count;
                }
                if current_draw_list_vertex_count > state.graph_vertex_count_max {
                    state.graph_vertex_count_max = current_draw_list_vertex_count;
                }

                // NOTE(yan): Some Casey vibes right here.
                for i in 0..GRAPH_LEN {
                    let graph_frame_build_max = if state.graph_frame_build_max == 0.0 {
                        1.0
                    } else {
                        state.graph_frame_build_max
                    };
                    ctrl.draw_rect(
                        false,
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 1.0 * height / 3.0,
                            0.3 * column_width,
                            0.3 * state.graph_frame_build[i] / graph_frame_build_max * height / 3.0,
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
                    ctrl.draw_rect(
                        false,
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 2.0 * height / 3.0,
                            0.3 * column_width,
                            0.3 * state.graph_command_count[i] as f32 / graph_command_count_max
                                * height
                                / 3.0,
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
                    ctrl.draw_rect(
                        false,
                        guise::Rect::new(
                            i as f32 * column_width,
                            height - 3.0 * height / 3.0,
                            0.3 * column_width,
                            0.3 * state.graph_vertex_count[i] as f32 / graph_vertex_count_max
                                * height
                                / 3.0,
                        ),
                        guise::Rect::ZERO,
                        if i == current_idx {
                            0x29a0b1ff
                        } else {
                            0x29a0b155
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

    if let Some(mut ctrl) = guise::begin_window(
        frame,
        line!(),
        theme,
        guise::Layout::Free,
        "1%",
        "1%",
        "23%",
        "48%",
    ) {
        let inner_extents = ctrl.inner_extents();
        ctrl.draw_rect(
            false,
            guise::Rect::new(0.0, 0.0, inner_extents.x, inner_extents.y),
            guise::Rect::UNIT,
            0xffffffff,
            0,
        );

        if let Some(mut ctrl) = guise::begin_window(
            frame,
            line!(),
            theme,
            guise::Layout::Vertical,
            5.0,
            5.0,
            100.0,
            50.0,
        ) {
            ctrl.draw_text(
                false,
                "Hello",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }
        if let Some(mut ctrl) = guise::begin_window(
            frame,
            line!(),
            theme,
            guise::Layout::Vertical,
            100.0,
            100.0,
            150.0,
            50.0,
        ) {
            ctrl.draw_text(
                false,
                "Traveller",
                guise::Align::Center,
                guise::Align::Center,
                guise::Wrap::Word,
                0x102030ff,
            );
            guise::end_window(frame);
        }
        if let Some(mut ctrl) = guise::begin_window(
            frame,
            line!(),
            theme,
            guise::Layout::Vertical,
            50.0,
            250.0,
            200.0,
            100.0,
        ) {
            ctrl.draw_text(
                false,
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

    if guise::begin_window(
        frame,
        line!(),
        theme,
        guise::Layout::Vertical,
        "1%",
        "51%",
        "23%",
        "48%",
    )
    .is_some()
    {
        guise::text_input(frame, line!(), theme, &mut state.text_input_text_heap);
        guise::text_input(frame, line!(), theme, &mut state.text_input_text_inline);
        if guise::button(frame, line!(), theme, "Clear") {
            state.text_input_text_heap.clear();
            state.text_input_text_inline.clear();
        }

        guise::end_window(frame);
    }
}
