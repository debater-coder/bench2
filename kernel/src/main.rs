#![no_std]
#![no_main]

use core::panic::PanicInfo;
use crate::gop_buffer::{Writer, WRITER};


mod gop_buffer;
mod debug_log;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().expect("Expected framebuffer");
    let (framebuffer_info, raw_framebuffer) = (framebuffer.info().clone(), framebuffer.buffer_mut());

    *WRITER.lock() = Some(unsafe { Writer::new(raw_framebuffer, framebuffer_info) });

    println!("Hello, World!");

    loop {}
}