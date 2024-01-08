use crate::io::drivers::apic::lapic_end_of_interrupt;
use crate::memory::gdt;
use crate::{print, println};
use lazy_static::lazy_static;
use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt[0x30].set_handler_fn(timer_handler);
        idt[0x41].set_handler_fn(keyboard_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)
        };

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {
        hlt()
    }
}

extern "x86-interrupt" fn keyboard_handler(_interrupt_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    print!("{}", scancode);

    unsafe { lapic_end_of_interrupt() };
}

extern "x86-interrupt" fn breakpoint_handler(interrupt_stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", interrupt_stack_frame);
}

extern "x86-interrupt" fn timer_handler(_interrupt_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe { lapic_end_of_interrupt() };
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: GPF\n{:#?}\nError Code: {:?}",
        stack_frame, error_code
    );
}
