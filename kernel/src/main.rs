#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let framebuffer = &mut boot_info.framebuffer.as_mut().expect("Expected framebuffer");
    let (framebuffer_info, raw_framebuffer) = (framebuffer.info().clone(), framebuffer.buffer_mut());

    for y in 0..framebuffer_info.height {
        for x in 0..framebuffer_info.width {
            let r = (y as f32 / framebuffer_info.height as f32 * 256.0) as u8;
            let g = (x as f32 / framebuffer_info.width as f32 * 256.0) as u8;
            let b = 128u8;
            raw_framebuffer[y * framebuffer_info.stride * 4 + x * 4] = b;
            raw_framebuffer[y * framebuffer_info.stride * 4 + x * 4 + 1] = g;
            raw_framebuffer[y * framebuffer_info.stride * 4 + x * 4 + 2] = r;
        }
    }

    loop {}
}