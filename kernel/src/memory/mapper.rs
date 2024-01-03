use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::VirtAddr;

pub static MAPPER: Mutex<Option<OffsetPageTable>> = Mutex::new(None);

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
