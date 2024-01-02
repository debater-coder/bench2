use crate::memory::{FRAME_ALLOCATOR, MAPPER};
use crate::virtual_addresses::HEAP_START;
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    let page_range = Page::range_inclusive(
        Page::containing_address(heap_start),
        Page::containing_address(heap_end),
    );

    for page in page_range {
        let mut mutex_guard = FRAME_ALLOCATOR.lock();
        let frame_allocator = mutex_guard.as_mut().unwrap();

        let mut mutex_guard = MAPPER.lock();
        let mapper = mutex_guard.as_mut().unwrap();

        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), HEAP_SIZE);
    }

    Ok(())
}
