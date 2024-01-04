#![no_std]
#![feature(asm_const)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

use crate::device_manager::DeviceManager;
use crate::gop_buffer::Writer;

extern crate alloc;

pub mod debug_log;
pub mod device_manager;
mod gop_buffer;
mod memory;

pub fn init(boot_info: &'static mut bootloader_api::BootInfo) -> DeviceManager {
    x86_64::instructions::interrupts::disable();

    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Expected framebuffer");

    let (framebuffer_info, raw_framebuffer) =
        (framebuffer.info().clone(), framebuffer.buffer_mut());

    unsafe { Writer::init(raw_framebuffer, framebuffer_info) };

    device_manager::interrupts::init_idt();

    let (frame_allocator, mapper) = memory::init(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("no physical memory offset"),
        &boot_info.memory_regions,
    );

    let device_manager = DeviceManager::new(
        frame_allocator,
        mapper,
        boot_info.rsdp_addr.into_option().expect("no rsdp") as usize,
    );

    x86_64::instructions::interrupts::enable();

    device_manager
}
