#![no_std]

//! # `include_programs`
//!
//! This crate includes ELF binaries from the host filesystem as raw bytes at
//! compile time.
//!
//! The `build.rs` script builds each program in the `programs`
//! directory. Then, when this crate is built, the binaries are included in this
//! one.
//!
//! New programs can be added to the `programs!` invocation with the format:
//!
//! ```
//! EXPORT_SYMBOL = "path/from/../../programs";
//! ```
//!
//! Crates that depend on this (e.g. the kernel) can then access byte-slices of
//! the program at `halogen_programs::EXPORT_SYMBOL`.

/// Include a single ELF with `include_bytes!`
macro_rules! include_program {
    ($name:expr, $id:ident) => {
        pub static $id: &[u8] =
            include_bytes!(concat!("../../programs/", $name, "/", $name, ".elf"));
    };
}

/// Declare programs for inclusion
///
/// ```
/// EXPORT_SYMBOL = "path/from/../../programs";
/// ```
macro_rules! programs {
    { $($id:ident = $name:expr);* $(;)* } => {
        $(include_program!($name, $id))*;
    }
}

programs! {
    HELLO = "hello";
}
