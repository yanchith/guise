use core::alloc::Allocator;
use core::fmt::Write;
use core::str::FromStr;

use arrayvec::ArrayString;

use crate::core::Frame;
use crate::widgets::{do_text_input_and_file_taxes, Theme};

// TODO(yan): float2_input, float3_input, float4_input
// TODO(yan): Consider adding a slider handle to float inputs and removing float sliders.

#[inline]
pub fn float_input<A>(frame: &mut Frame<A>, id: u32, value: &mut f32, label: &str) -> bool
where
    A: Allocator + Clone,
{
    float_input_with_min_max_precision_theme(
        frame,
        id,
        value,
        label,
        f32::MIN,
        f32::MAX,
        3,
        &Theme::DEFAULT,
    )
}

#[inline]
pub fn float_input_with_min_max_precision<A>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut f32,
    label: &str,
    min: f32,
    max: f32,
    precision: u16,
) -> bool
where
    A: Allocator + Clone,
{
    float_input_with_min_max_precision_theme(
        frame,
        id,
        value,
        label,
        min,
        max,
        precision,
        &Theme::DEFAULT,
    )
}

#[inline]
pub fn float_input_with_theme<A>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut f32,
    label: &str,
    theme: &Theme,
) -> bool
where
    A: Allocator + Clone,
{
    float_input_with_min_max_precision_theme(frame, id, value, label, f32::MIN, f32::MAX, 3, theme)
}

#[inline]
pub fn float_input_with_min_max_precision_theme<A>(
    frame: &mut Frame<A>,
    id: u32,
    value: &mut f32,
    label: &str,
    min: f32,
    max: f32,
    precision: u16,
    theme: &Theme,
) -> bool
where
    A: Allocator + Clone,
{
    let mut buf: ArrayString<128> = ArrayString::new();

    // TODO(yan): Current approach draws a value first, and only then applies
    // parsing and clamping rejections. This looks jumpy onscreen. For this to
    // work well, we'd have to do drawing in here.
    let _ = write!(buf, "{:.1$}", value, usize::from(precision));
    if do_text_input_and_file_taxes::<_, _, &str>(
        frame,
        id,
        &mut buf,
        label,
        None,
        Some(&float_filter),
        &[],
        theme,
    ) {
        match f32::from_str(&buf) {
            Ok(mut new_value) => {
                new_value = f32::clamp(new_value, min, max);

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

fn float_filter(c: char) -> Option<char> {
    if c == '.' || c == '+' || c == '-' || c.is_ascii_digit() {
        Some(c)
    } else {
        None
    }
}
