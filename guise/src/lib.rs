#![no_std]
#![allow(clippy::too_many_arguments)]

// TODO(yan): Don't even depend on alloc.
extern crate alloc;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[macro_use]
mod macros;

mod convert;
mod core;
mod widgets;

pub use crate::core::*;
pub use crate::widgets::*;

// This should not compile if this crate doesn't accidentally depend on std.
//
// fn no_std_compile_test() {
//     f32::sin(1.0);
// }
