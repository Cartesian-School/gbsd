use core::mem::size_of;
use x86_64::structures::idt::InterruptDescriptorTable;

/// Global IDT that will be used
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Initialize IDT with exception and interrupt handlers
pub fn init_idt() {
    unsafe {
        // Exceptions (0-31)
        IDT.divide_error.set_handler_fn(divide_error_handler);
        IDT.debug.set_handler_fn(debug_handler);
        IDT.non_maskable_interrupt.set_handler_fn(nmi_handler);
        IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.overflow.set_handler_fn(overflow_handler);
        IDT.bound_range_exceeded.set_handler_fn(bound_range_handler);
        IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        IDT.device_not_available.set_handler_fn(device_not_available_handler);
        // Double fault needs a special stack
        // IDT.double_fault.set_handler_fn(double_fault_handler);
        IDT.invalid_tss.set_handler_fn(invalid_tss_handler);
        IDT.segment_not_present.set_handler_fn(segment_not_present_handler);
        IDT.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        IDT.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.x87_floating_point.set_handler_fn(floating_point_handler);
        IDT.alignment_check.set_handler_fn(alignment_check_handler);
        IDT.machine_check.set_handler_fn(machine_check_handler);
        IDT.simd_floating_point.set_handler_fn(simd_floating_point_handler);

        // Load the IDT
        IDT.load();
    }
}

// Exception handlers
extern "x86-interrupt" fn divide_error_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Divide Error!");
}

extern "x86-interrupt" fn debug_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Debug Exception!");
}

extern "x86-interrupt" fn nmi_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Non-Maskable Interrupt!");
}

extern "x86-interrupt" fn breakpoint_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    // Breakpoint - this is expected during debugging
}

extern "x86-interrupt" fn overflow_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Overflow!");
}

extern "x86-interrupt" fn bound_range_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Bound Range Exceeded!");
}

extern "x86-interrupt" fn invalid_opcode_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Invalid Opcode!");
}

extern "x86-interrupt" fn device_not_available_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("Device Not Available!");
}

extern "x86-interrupt" fn invalid_tss_handler(_frame: x86_64::structures::idt::InterruptStackFrame, _code: u64) {
    panic!("Invalid TSS!");
}

extern "x86-interrupt" fn segment_not_present_handler(_frame: x86_64::structures::idt::InterruptStackFrame, _code: u64) {
    panic!("Segment Not Present!");
}

extern "x86-interrupt" fn stack_segment_fault_handler(_frame: x86_64::structures::idt::InterruptStackFrame, _code: u64) {
    panic!("Stack Segment Fault!");
}

extern "x86-interrupt" fn general_protection_fault_handler(_frame: x86_64::structures::idt::InterruptStackFrame, _code: u64) {
    panic!("General Protection Fault!");
}

extern "x86-interrupt" fn page_fault_handler(_frame: x86_64::structures::idt::InterruptStackFrame, _code: u64) {
    panic!("Page Fault!");
}

extern "x86-interrupt" fn floating_point_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("x87 Floating Point Exception!");
}

extern "x86-interrupt" fn alignment_check_handler(_frame: x86_64::structures::idt::InterruptStackFrame, _code: u64) {
    panic!("Alignment Check!");
}

extern "x86-interrupt" fn machine_check_handler(_frame: x86_64::structures::idt::InterruptStackFrame) -> ! {
    panic!("Machine Check!");
}

extern "x86-interrupt" fn simd_floating_point_handler(_frame: x86_64::structures::idt::InterruptStackFrame) {
    panic!("SIMD Floating Point Exception!");
}


