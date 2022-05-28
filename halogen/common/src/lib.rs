//! This crate provides data structures and types for the Halogen kernel. These
//! are mostly independent of architecture and kernel implementation and can
//! therefore be tested on the host machine.

#![feature(pointer_is_aligned)]
#![feature(const_mut_refs)]
// Only use the standard library for testing.
#![cfg_attr(not(test), no_std)]

// Only use the `alloc` crate if not testing and the "alloc" feature is enabled.
#[cfg(all(feature = "alloc", not(test)))]
extern crate alloc;

/// Math and bit-manip helper functions.
pub mod math;
/// Data structures and algorithms for memory-management.
pub mod mem;

/// Data structures and algorithms for task scheduling.
#[cfg(feature = "alloc")]
pub mod sched;
