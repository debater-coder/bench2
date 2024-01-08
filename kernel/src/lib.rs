#![no_std]
#![feature(asm_const)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

use crate::gop_buffer::Writer;

extern crate alloc;

pub mod debug_log;
mod gop_buffer;
pub mod io;
mod memory;

pub unsafe fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    init_inner(boot_info)
}

fn init_inner(boot_info: &'static mut bootloader_api::BootInfo) {
    x86_64::instructions::interrupts::disable();

    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Expected framebuffer");

    let (framebuffer_info, raw_framebuffer) =
        (framebuffer.info().clone(), framebuffer.buffer_mut());

    unsafe { Writer::init(raw_framebuffer, framebuffer_info) };

    io::interrupts::init_idt();

    let mut memory_allocator = unsafe {
        memory::init(
            boot_info
                .physical_memory_offset
                .into_option()
                .expect("no physical memory offset"),
            &boot_info.memory_regions,
        )
    };

    unsafe {
        io::init(
            &mut memory_allocator,
            boot_info.rsdp_addr.into_option().expect("no rsdp") as usize,
        );
    }

    x86_64::instructions::interrupts::enable();
}
