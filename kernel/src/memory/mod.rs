use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub mod allocator;
pub mod gdt;
pub mod virtual_addresses;

pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);
pub static MAPPER: Mutex<Option<OffsetPageTable>> = Mutex::new(None);

pub struct BootInfoFrameAllocator {
    next: usize,
    memory_regions: &'static MemoryRegions,
}

// This is safe because there will only ever be one FrameAllocator
unsafe impl Send for BootInfoFrameAllocator {}
unsafe impl Sync for BootInfoFrameAllocator {}

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
    pub fn init(memory_regions: &'static MemoryRegions) {
        if FRAME_ALLOCATOR.lock().is_some() {
            panic!("Frame allocator must only be initialised once");
        }
        *FRAME_ALLOCATOR.lock() = Some(BootInfoFrameAllocator {
            next: 0,
            memory_regions,
        });
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.available_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub fn init(physical_memory_offset: VirtAddr) {
    if MAPPER.lock().is_some() {
        panic!("Mapper must only be initialised once");
    }
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        *MAPPER.lock() = Some(OffsetPageTable::new(level_4_table, physical_memory_offset));
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table, _) = Cr3::read();

    let address = physical_memory_offset + level_4_table.start_address().as_u64();
    let pointer: *mut PageTable = address.as_mut_ptr();

    &mut *pointer
}
