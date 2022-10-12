#![no_std]
#![no_main]

#![deny(unsafe_op_in_unsafe_fn)]

// Transmute slices if this feature doesn't get stabilized
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_uninit_array)]
#![feature(int_roundings)]
#![feature(is_sorted)]
#![feature(atomic_mut_ptr)]
#![feature(sync_unsafe_cell)]

pub mod common;
pub mod boot;
pub mod arch;
pub mod allocator;

use core::{panic::PanicInfo, arch::asm};

// Get terminal, setup early logging
// Get memory map, setup global allocator / kmalloc
// Get framebuffer
// Call main
// Load fonts, setup framebuffer terminal
// Setup IRQs
// ...

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    boot_println!("Panic! {}", _info);
    loop {
        unsafe {
            asm!(
                "cli",
                "hlt"
            );
        }
    }
}
