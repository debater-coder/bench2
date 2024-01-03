use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

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
