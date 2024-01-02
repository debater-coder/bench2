use crate::memory::MAPPER;
use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;

#[derive(Clone)]
pub struct BenchAcpiHandler;

impl BenchAcpiHandler {
    pub fn new() -> Self {
        BenchAcpiHandler
    }
}

impl AcpiHandler for BenchAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let mut mutex_guard = MAPPER.lock();
        let mapper = mutex_guard.as_mut().unwrap();

        PhysicalMapping::new(
            physical_address,
            NonNull::new((mapper.phys_offset() + physical_address).as_mut_ptr()).unwrap(),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // FIXME: Add region unmapping to frame allocator to avoid memory leaks
    }
}
