/// # Handles IDT
/// Even if a device uses less than 16 IRQs we still reserve 16 to keep things aligned nicely (and for prioritisation)
///
/// Interrupts 00-1F are reserved for exceptions
/// Interrupts 20-2F are spurious interrupts from the legacy PIC
/// Interrupts 30-3F are Local APIC LVT interrupts (CMCI, Timer, Thermal Monitor, Performance Counter, LINT0, LINT1 and
///     Error) respectively
/// Interrupt 40-4F are ISA IRQs with the interrupt number corresponding with the IRQ (eg. 0 is PIC, 1 is PS/2 Keyboard etc.)
/// Interrupt 50-5F are PCI interrupts (not yet implemented)
///
/// Interrupt 80 is for syscalls from userspace (not yet implemented)
///
/// Interrupt FF is spurious interrupt (currently from LAPIC only)
use crate::memory::gdt;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        {
            use exception_handlers::*;

            idt.divide_error.set_handler_fn(divide_error);
            idt.debug.set_handler_fn(debug);
            idt.non_maskable_interrupt
                .set_handler_fn(non_maskable_interrupt);
            idt.breakpoint.set_handler_fn(breakpoint_handler);
            idt.overflow.set_handler_fn(overflow);
            idt.bound_range_exceeded
                .set_handler_fn(bound_range_exceeded);
            idt.invalid_opcode.set_handler_fn(invalid_opcode);
            idt.device_not_available
                .set_handler_fn(device_not_available);

            unsafe {
                idt.double_fault
                    .set_handler_fn(double_fault)
                    .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)
            };

            idt.invalid_tss.set_handler_fn(invalid_tss);
            idt.segment_not_present.set_handler_fn(segment_not_present);
            idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
            idt.general_protection_fault
                .set_handler_fn(general_protection_fault);
            idt.page_fault.set_handler_fn(page_fault);
            idt.x87_floating_point.set_handler_fn(x87_floating_point);
            idt.alignment_check.set_handler_fn(alignment_check);
            idt.machine_check.set_handler_fn(machine_check);
            idt.simd_floating_point.set_handler_fn(simd_floating_point);
            idt.virtualization.set_handler_fn(virtualization);
        }

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub(super) mod exception_handlers {
    use x86_64::registers::control::Cr2;
    use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

    pub(super) extern "x86-interrupt" fn divide_error(_interrupt_stack_frame: InterruptStackFrame) {
        panic!("[CPU Exception] Divide Error");
    }

    pub(super) extern "x86-interrupt" fn debug(_interrupt_stack_frame: InterruptStackFrame) {}
    pub(super) extern "x86-interrupt" fn non_maskable_interrupt(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] Non-Maskable Interrupt")
    }

    pub(super) extern "x86-interrupt" fn breakpoint_handler(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
    }
    pub(super) extern "x86-interrupt" fn overflow(_interrupt_stack_frame: InterruptStackFrame) {
        panic!("[CPU Exception] Overflow")
    }

    pub(super) extern "x86-interrupt" fn bound_range_exceeded(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] Bound Range Exceeded")
    }

    pub(super) extern "x86-interrupt" fn invalid_opcode(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] Invalid Opcode")
    }

    pub(super) extern "x86-interrupt" fn device_not_available(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] Device Not Available")
    }

    pub(super) extern "x86-interrupt" fn double_fault(
        _interrupt_stack_frame: InterruptStackFrame,
        _error_code: u64,
    ) -> ! {
        panic!("[CPU Exception] Device Not Available")
    }

    pub(super) extern "x86-interrupt" fn invalid_tss(
        _interrupt_stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        panic!("[CPU Exception] Invalid TSS {:?}", error_code)
    }

    pub(super) extern "x86-interrupt" fn segment_not_present(
        _interrupt_stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        panic!("[CPU Exception] Segment Not Present {:?}", error_code)
    }

    pub(super) extern "x86-interrupt" fn stack_segment_fault(
        _interrupt_stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        panic!("[CPU Exception] Stack Segment Fault {:?}", error_code)
    }

    pub(super) extern "x86-interrupt" fn general_protection_fault(
        _interrupt_stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        panic!("[CPU Exception] General Protection Fault {:?}", error_code)
    }

    pub(super) extern "x86-interrupt" fn page_fault(
        _interrupt_stack_frame: InterruptStackFrame,
        error_code: PageFaultErrorCode,
    ) {
        panic!(
            "[CPU Exception] Page Fault on address {:?}, {:?}",
            Cr2::read(),
            error_code
        )
    }

    pub(super) extern "x86-interrupt" fn x87_floating_point(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] x87 Floating Point Error")
    }

    pub(super) extern "x86-interrupt" fn alignment_check(
        _interrupt_stack_frame: InterruptStackFrame,
        _error_code: u64,
    ) {
        panic!("[CPU Exception] Alignment Check")
    }

    pub(super) extern "x86-interrupt" fn machine_check(
        _interrupt_stack_frame: InterruptStackFrame,
    ) -> ! {
        panic!("[CPU Exception] Machine Check")
    }

    pub(super) extern "x86-interrupt" fn simd_floating_point(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] SIMD Floating Point Error")
    }

    pub(super) extern "x86-interrupt" fn virtualization(
        _interrupt_stack_frame: InterruptStackFrame,
    ) {
        panic!("[CPU Exception] Virtualization Error")
    }
}
