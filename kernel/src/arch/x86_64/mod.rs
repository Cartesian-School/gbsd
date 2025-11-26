// kernel/src/arch/x86_64/mod.rs
// x86_64 architecture-specific code

pub mod idt;

pub fn kernel_main() -> ! {
    idt::init_idt();

    crate::serial::write_str("[kernel] IDT initialized\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

