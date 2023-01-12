mod button;
mod checkbox;
mod dropdown;
mod float_slider;
mod int_slider;
mod panel;
mod separator;
mod size;
mod text;
mod text_input;
mod theme;
mod tooltip;
mod window;

pub use button::*;
pub use checkbox::*;
pub use dropdown::*;
pub use float_slider::*;
pub use int_slider::*;
pub use panel::*;
pub use separator::*;
pub use size::*;
pub use text::*;
pub use text_input::*;
pub use theme::*;
pub use tooltip::*;
pub use window::*;

// TODO(yan): Widget API surface:
//
// Over time, as I've used this, I don't really like the builder style too much,
// and the plain function calls like this are preferrable to me:
//
//     guise::dropdown(
//         frame,
//         line!(),
//         "Damage Type",
//         DAMAGE_TYPES,
//         &mut state.dropdown1_selected_option,
//     );
//
// With panels and windows (and whatever else we have that can have child
// controls), we could do an if-let with en explicit guard:
//
//     if let Some(w) = guise::begin_window(frame, line!(), "1%", "51%", "39%", "48%") {
//         ... code ...
//         w.end();
//     }
//
// For controls that need more control, we can do more specific functions, like:
//
//     // Basic
//     guise::text_input(
//         frame,
//         line!(),
//         &mut state.text_input_heap,
//         "Heap String",
//     );
//
//     // Advanced
//     guise::text_input_with_callback(
//         frame,
//         line!(),
//         &mut state.text_input_heap,
//         "Heap String",
//         |data, _| match data.action {
//             guise::InputTextAction::None => (),
//             guise::InputTextAction::Submit => state.text_input_submit_count += 1,
//             guise::InputTextAction::Cancel => state.text_input_cancel_count += 1,
//         },
//     );
//
//     // Expert-level
//     guise::text_input_with_options(
//         frame,
//         line!(),
//         &mut state.text_input_heap,
//         "Heap String",
//         &theme,
//         &options,
//     );
