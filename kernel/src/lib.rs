#![no_std]
#![feature(asm_const)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

extern crate alloc;

pub mod debug_log;
pub mod io;
mod memory;

use io::drivers::display::gop_buffer::Writer;
use io::framebuffer;

pub unsafe fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    init_inner(boot_info)
}

fn init_inner(boot_info: &'static mut bootloader_api::BootInfo) {
    x86_64::instructions::interrupts::disable();

    framebuffer::init(
        boot_info
            .framebuffer
            .as_mut()
            .expect("Expected framebuffer"),
    );

    unsafe { Writer::init() };

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
