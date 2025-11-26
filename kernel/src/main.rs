#![no_std]
#![no_main]

mod task;
mod syscall;
#[cfg(target_arch = "x86_64")]
mod arch;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // инициализация системного стека, IDT, GDT
    arch::idt::init_idt();
    task::init_tasks();

    // переход к init_server
    crate::arch::x86_64::jump_to_userspace();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
