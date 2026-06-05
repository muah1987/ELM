#![no_std]

extern crate alloc; // CRITICAL: Exposes the allocator to all sub-modules

pub mod world;
pub mod memory;
pub mod sensors;
pub mod prediction;
