use crate::memory::frame_allocator::BootInfoFrameAllocator;
use crate::memory::heap_allocator::init_heap;
use bootloader_api::info::MemoryRegions;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;

pub(crate) mod frame_allocator;
pub(crate) mod gdt;
mod heap_allocator;
pub(crate) mod mapper;
pub(crate) mod virtual_addresses;

pub fn init(
    physical_memory_offset: u64,
    memory_regions: &'static MemoryRegions,
) -> (BootInfoFrameAllocator, OffsetPageTable<'static>) {
    gdt::init();

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(memory_regions) };
    let mut mapper = unsafe { mapper::new(VirtAddr::new(physical_memory_offset)) };

    init_heap(&mut frame_allocator, &mut mapper).expect("heap initialisation failed");

    (frame_allocator, mapper)
}
