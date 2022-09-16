#![no_std]
#![feature(allocator_api)]
#![feature(const_trait_impl)]
#![feature(generic_associated_types)]
#![allow(clippy::collapsible_if)] // tag: style
#![allow(clippy::comparison_chain)] // tag: style
#![allow(clippy::len_without_is_empty)] // tag: style
#![allow(clippy::new_without_default)] // tag: style
#![allow(clippy::should_implement_trait)] // tag: style
#![allow(clippy::too_many_arguments)] // tag: style

// TODO(yan): Don't even depend on alloc, if we ever have the time to write our
// own collections, or Vec/hashbrown becomes libcore.
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
