use crate::io::bench_acpi::BenchAcpiHandler;
use crate::memory::MemoryAllocator;
use acpi::AcpiTables;

mod bench_acpi;
mod drivers;
pub(crate) mod interrupts;

pub(crate) unsafe fn init(memory_allocator: &mut MemoryAllocator, rsdp_addr: usize) {
    let acpi_handler = BenchAcpiHandler::new(memory_allocator.phys_offset());

    let acpi_tables =
        unsafe { AcpiTables::from_rsdp(acpi_handler, rsdp_addr).expect("rsdp init failed") };
    let platform_info = acpi_tables.platform_info().unwrap();

    drivers::apic::Apic::new(memory_allocator, &platform_info.interrupt_model);
}
