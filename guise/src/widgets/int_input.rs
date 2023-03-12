use core::alloc::Allocator;
use core::fmt::Write;
use core::str::FromStr;

use arrayvec::ArrayString;

use crate::core::Frame;
use crate::widgets::{text_input_with_theme, Theme};

// TODO(yan): int2_input, int3_input, int4_input
// TODO(yan): Consider adding a slider handle to int inputs and removing int sliders.

#[inline]
pub fn int_input<A>(frame: &mut Frame<A>, id: u32, value: &mut i32, label: &str) -> bool
where
    A: Allocator + Clone,
{
    int_input_with_min_max_theme(frame, id, value, label, i32::MIN, i32::MAX, &Theme::DEFAULT)
}

#[inline]
pub fn int_input_with_min_max<A>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut i32,
    label: &str,
    min: i32,
    max: i32,
) -> bool
where
    A: Allocator + Clone,
{
    int_input_with_min_max_theme(frame, id, value, label, min, max, &Theme::DEFAULT)
}

#[inline]
pub fn int_input_with_theme<A>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut i32,
    label: &str,
    theme: &Theme,
) -> bool
where
    A: Allocator + Clone,
{
    int_input_with_min_max_theme(frame, id, value, label, i32::MIN, i32::MAX, theme)
}

#[inline]
pub fn int_input_with_min_max_theme<A>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut i32,
    label: &str,
    min: i32,
    max: i32,
    theme: &Theme,
) -> bool
where
    A: Allocator + Clone,
{
    let mut buf: ArrayString<128> = ArrayString::new();

    // TODO(yan): Current approach draws a value first, and only then applies
    // parsing and clamping rejections. This looks jumpy onscreen. Add a
    // rejection callback to text input, so that we can reject a value before it
    // is displayed.
    let _ = write!(buf, "{value}");
    if text_input_with_theme(frame, id, &mut buf, label, theme) {
        match i32::from_str(&buf) {
            Ok(mut new_value) => {
                new_value = i32::clamp(new_value, min, max);

                if *value != new_value {
                    *value = new_value;
                    return true;
                }
            }
            Err(_) => (),
        }
    }

    false
}
