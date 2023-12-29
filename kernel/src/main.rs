#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod tty;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let framebuffer = &mut boot_info.framebuffer.as_mut().expect("Expected framebuffer");
    let (framebuffer_info, raw_framebuffer) = (framebuffer.info().clone(), framebuffer.buffer_mut());

    tty::print_something(raw_framebuffer, framebuffer_info);
    loop {}
}