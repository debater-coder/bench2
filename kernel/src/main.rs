#![no_std]
#![no_main]

use core::panic::PanicInfo;
use crate::gop_buffer::Writer;
use core::fmt::Write;


mod gop_buffer;
mod debug_log;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().expect("Expected framebuffer");
    let (framebuffer_info, raw_framebuffer) = (framebuffer.info().clone(), framebuffer.buffer_mut());

    let mut writer = unsafe { Writer::new(raw_framebuffer, framebuffer_info) };

    write!(writer, "Hello, World!").unwrap();

    loop {}
}