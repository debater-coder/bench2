#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use crate::gop_buffer::Writer;
use bootloader_api::config::Mapping;
use bootloader_api::BootloaderConfig;
use core::panic::PanicInfo;

mod debug_log;
mod gdt;
mod gop_buffer;
mod interrupts;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
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

    gdt::init();
    interrupts::init_idt();

    println!("BenchOS 0.1.0");
    println!(
        "Framebuffer {:?}x{:?}",
        framebuffer_info.width, framebuffer_info.height
    );

    println!("It did not crash!");

    loop {
        x86_64::instructions::hlt();
    }
}
