use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub struct BootInfoFrameAllocator {
    next: usize,
    memory_regions: &'static MemoryRegions,
}

impl BootInfoFrameAllocator {
    fn available_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let available_memory_regions = self
            .memory_regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable);

        let available_frames = available_memory_regions
            .clone()
            .map(|region| region.start..region.end)
            .flatten()
            .filter(|addr| (addr & 0xfff) == 0)
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));

        available_frames
    }

    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            next: 0,
            memory_regions,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.available_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// # Safety
/// This function can only be called once (to avoid multiple &mut references) and the physical
/// memory is mapped at the given offset.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table, _) = Cr3::read();

    let address = physical_memory_offset + level_4_table.start_address().as_u64();
    let pointer: *mut PageTable = address.as_mut_ptr();

    &mut *pointer
}
