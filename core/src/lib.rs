//! Hardware accelerated routines for single substring search

#![cfg_attr(not(test), no_std)]

#[cfg(not(any(target_pointer_width = "64", target_endian = "little")))]
compile_error!("ashwa is only supported on 64-bit targets");

mod one;

pub use one::search_one;
