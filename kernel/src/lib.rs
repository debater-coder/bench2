#![no_std]
#![feature(asm_const)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

use crate::allocator::init_heap;
use crate::bench_acpi::BenchAcpiHandler;
use crate::gop_buffer::Writer;
use crate::memory::BootInfoFrameAllocator;
use acpi::AcpiTables;
use x86_64::VirtAddr;

extern crate alloc;

mod allocator;
mod apic;
mod bench_acpi;
pub mod debug_log;
mod gdt;
mod gop_buffer;
mod interrupts;
mod memory;
mod virtual_addresses;

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    x86_64::instructions::interrupts::disable();

    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Expected framebuffer");

    let (framebuffer_info, raw_framebuffer) =
        (framebuffer.info().clone(), framebuffer.buffer_mut());

    unsafe { Writer::init(raw_framebuffer, framebuffer_info) };

    gdt::init();
    interrupts::init_idt();
    BootInfoFrameAllocator::init(&boot_info.memory_regions);
    memory::init(VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("Expected memory offset"),
    ));

    init_heap().expect("heap initialisation failed");

    let acpi_handler = BenchAcpiHandler::new();

    let acpi_tables = unsafe {
        AcpiTables::from_rsdp(
            acpi_handler,
            boot_info
                .rsdp_addr
                .into_option()
                .expect("no rsdp")
                .try_into()
                .unwrap(),
        )
        .expect("rsdp init failed")
    };
    let platform_info = acpi_tables.platform_info().unwrap();

    apic::init(&platform_info.interrupt_model);
    x86_64::instructions::interrupts::enable();
}
