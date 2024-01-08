use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::VirtAddr;

#[derive(Clone)]
pub struct BenchAcpiHandler {
    phys_offset: VirtAddr,
}

impl BenchAcpiHandler {
    pub fn new(phys_offset: VirtAddr) -> Self {
        BenchAcpiHandler { phys_offset }
    }
}

impl AcpiHandler for BenchAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        PhysicalMapping::new(
            physical_address,
            NonNull::new((self.phys_offset + physical_address).as_mut_ptr()).unwrap(),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // FIXME: Add region unmapping to frame allocator to avoid memory leaks
    }
}
