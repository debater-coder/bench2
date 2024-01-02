#![feature(asm_const)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

use crate::allocator::init_heap;
use crate::bench_acpi::BenchAcpiHandler;
use crate::gop_buffer::Writer;
use crate::memory::BootInfoFrameAllocator;
use acpi::{AcpiTables, InterruptModel};
use bootloader_api::config::Mapping;
use bootloader_api::BootloaderConfig;
use core::panic::PanicInfo;
use x86_64::VirtAddr;

mod allocator;
mod apic;
mod bench_acpi;
mod debug_log;
mod gdt;
mod gop_buffer;
mod interrupts;
mod memory;
mod virtual_addresses;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(0x20000000000));
    config
};

bootloader_api::entry_point!(kernel_early, config = &BOOTLOADER_CONFIG);

fn kernel_early(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Expected framebuffer");

    let (framebuffer_info, raw_framebuffer) =
        (framebuffer.info().clone(), framebuffer.buffer_mut());

    unsafe { Writer::init(raw_framebuffer, framebuffer_info) };

    println!("Initialising GDT...");
    gdt::init();

    println!("Initialising interrupts...");
    interrupts::init_idt();

    println!("Initialising frame allocator...");
    BootInfoFrameAllocator::init(&boot_info.memory_regions);

    println!("Initialising page table mapper...");

    memory::init(VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("Expected memory offset"),
    ));

    println!(
        "Offset {:x}",
        boot_info.physical_memory_offset.into_option().unwrap()
    );

    println!("Initialising heap...");
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

    match platform_info.interrupt_model {
        InterruptModel::Apic(apic_info) => {
            println!("Local apic address: {:x}", apic_info.local_apic_address);
        }
        _ => {
            panic!("No APIC")
        }
    }

    println!("Initialising APIC...");
    apic::init();

    println!("BenchOS 0.1.0");
    println!(
        "Framebuffer {:?}x{:?}",
        framebuffer_info.width, framebuffer_info.height
    );

    println!("It did not crash!");

    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
