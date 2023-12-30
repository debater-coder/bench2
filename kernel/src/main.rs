#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use crate::gop_buffer::Writer;
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

bootloader_api::entry_point!(kernel_early);

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

    loop {
        x86_64::instructions::hlt();
    }
}
