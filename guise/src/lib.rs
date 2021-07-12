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
