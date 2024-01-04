use crate::device_manager::bench_acpi::BenchAcpiHandler;
use crate::memory::frame_allocator::BootInfoFrameAllocator;
use acpi::AcpiTables;
use x86_64::structures::paging::OffsetPageTable;

mod apic;
mod bench_acpi;
pub(crate) mod interrupts;

pub struct DeviceManager {
    frame_allocator: BootInfoFrameAllocator,
    mapper: OffsetPageTable<'static>,
}

impl DeviceManager {
    pub(crate) fn new(
        mut frame_allocator: BootInfoFrameAllocator,
        mut mapper: OffsetPageTable<'static>,
        rsdp_addr: usize,
    ) -> Self {
        let acpi_handler = BenchAcpiHandler::new(mapper.phys_offset());

        let acpi_tables =
            unsafe { AcpiTables::from_rsdp(acpi_handler, rsdp_addr).expect("rsdp init failed") };
        let platform_info = acpi_tables.platform_info().unwrap();

        apic::init(
            &mut frame_allocator,
            &mut mapper,
            &platform_info.interrupt_model,
        );

        DeviceManager {
            frame_allocator,
            mapper,
        }
    }
}
