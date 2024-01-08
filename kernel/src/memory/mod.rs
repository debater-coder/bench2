use crate::memory::frame_allocator::BootInfoFrameAllocator;
use crate::memory::heap_allocator::init_heap;
use bootloader_api::info::MemoryRegions;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub(crate) mod frame_allocator;
pub(crate) mod gdt;
mod heap_allocator;
pub(crate) mod mapper;
pub(crate) mod virtual_addresses;

pub struct MemoryAllocator(pub BootInfoFrameAllocator, pub OffsetPageTable<'static>);

impl MemoryAllocator {
    pub fn phys_offset(&self) -> VirtAddr {
        self.1.phys_offset()
    }

    /// phys_address and virt_address should be aligned to 4KiB boundary
    pub unsafe fn map_page_containing_address(&mut self, phys_address: PhysAddr, virt_addr: VirtAddr, flags: PageTableFlags) {
        let (frame_allocator, mapper) = (&mut self.0, &mut self.1);

        let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_address);

        let page: Page<Size4KiB> = Page::containing_address(virt_addr);

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .unwrap()
                .flush();
        }
    }
}

pub unsafe fn init(
    physical_memory_offset: u64,
    memory_regions: &'static MemoryRegions,
) -> MemoryAllocator {
    gdt::init();

    let mut frame_allocator = BootInfoFrameAllocator::new(memory_regions);
    let mut mapper = mapper::new(VirtAddr::new(physical_memory_offset));

    init_heap(&mut frame_allocator, &mut mapper).expect("heap initialisation failed");

    MemoryAllocator(frame_allocator, mapper)
}
