use core::alloc::Allocator;
use core::fmt::Write;
use core::str::FromStr;

use arrayvec::ArrayString;

use crate::core::Frame;
use crate::widgets::{text_input_with_theme, Theme};

#[inline]
pub fn int_input<A>(frame: &mut Frame<A>, id: u32, value: &mut i32, label: &str) -> bool
where
    A: Allocator + Clone,
{
    int_input_with_theme(frame, id, value, label, &Theme::DEFAULT)
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
    let mut buf: ArrayString<128> = ArrayString::new();

    let _ = write!(buf, "{value}");
    if text_input_with_theme(frame, id, &mut buf, label, theme) {
        match i32::from_str(&buf) {
            Ok(new_value) => {
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

// TODO(yan): int2, int3, int4
// TODO(yan): min/max
