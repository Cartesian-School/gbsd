#![no_std]
#![no_main]

extern crate x86_64;
extern crate uart_16550;
extern crate alloc;
extern crate spin;

use core::panic::PanicInfo;

// VGA text buffer module
mod vga;
mod serial;
mod panic;
mod arch;
mod memory;

// Kernel infrastructure
pub mod error;
pub mod globals;
pub mod ipc;
pub mod syscall;

// Unit tests
#[cfg(test)]
mod error_tests;

#[cfg(test)]
mod ipc_tests;

/// Kernel entry point called by the bootloader.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize subsystems
    vga::clear_screen();
    vga::print_str("GBSD kernel starting...\n");

    serial::init();
    serial::write_str("Serial initialized.\n");

    memory::init();

    vga::print_str("Initialization complete.\n");

    // Enter architecture-specific main loop
    arch::kernel_main()
}

/// Required panic handler (delegated to panic.rs).
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::panic_handler(info)
}
