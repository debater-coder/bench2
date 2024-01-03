use crate::memory::frame_allocator::BootInfoFrameAllocator;
use crate::memory::heap_allocator::init_heap;
use bootloader_api::info::MemoryRegions;
use x86_64::VirtAddr;

pub(crate) mod frame_allocator;
pub(crate) mod gdt;
mod heap_allocator;
pub(crate) mod mapper;
pub(crate) mod virtual_addresses;

pub fn init(physical_memory_offset: u64, memory_regions: &'static MemoryRegions) {
    gdt::init();
    BootInfoFrameAllocator::init(memory_regions);
    mapper::init(VirtAddr::new(physical_memory_offset));
    init_heap().expect("heap initialisation failed");
}
