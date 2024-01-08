pub(super) mod ioapic;
pub(super) mod lapic;

pub use lapic::lapic_end_of_interrupt;

use crate::io::drivers::apic::lapic::{Lapic, TimerDivideConfig};
use crate::memory::MemoryAllocator;
use acpi::InterruptModel;
use alloc::alloc::Global;
use ioapic::IoApic;
use pic8259::ChainedPics;
use x86_64::instructions::port::Port;
use x86_64::registers::model_specific::Msr;

#[allow(dead_code)]
enum IsaIrq {
    PitTimer = 0,
    Keyboard = 1,
    Com2 = 3,
    Com1 = 4,
    Lpt2 = 5,
    FloppyDisk = 6,
    Isa7 = 7,
    Rtc = 8,
    Isa9 = 9,
    Isa10 = 10,
    Isa11 = 11,
    Mouse = 12,
    Isa13 = 13,
    PrimaryAta = 14,
    SecondaryAta = 15,
}

pub struct Apic;

impl Apic {
    pub fn new(
        memory_allocator: &mut MemoryAllocator,
        interrupt_model: &InterruptModel<Global>,
    ) -> Self {
        // The following section uses the overall steps from: https://blog.wesleyac.com/posts/ioapic-interrupts

        let (ioapics, interrupt_source_overrides) = match interrupt_model {
            InterruptModel::Apic(apic_info) => {
                (&apic_info.io_apics, &apic_info.interrupt_source_overrides)
            }
            _ => {
                panic!("interrupt model is not apic")
            }
        };

        // Both QEMU and my test system only have one IOAPIC so this is fine for now.
        let ioapic = &ioapics[0];

        // Step 1: Disable the PIC.
        unsafe {
            let mut pics = ChainedPics::new(0x20, 0x28);
            // This remaps it so that when we have a spurious interrupt it doesn't mess us up
            pics.initialize();

            pics.disable();
        }

        // Step 2: Set IMCR (IMCR is optional if PIC mode is not implemented so it is uncommon on modern systems)
        let mut imcr = IMCR::new();
        imcr.enable_symmetric_io_mode();

        // Step 3: Configure the "Spurious Interrupt Vector Register" of the Local APIC to 0xFF
        let mut lapic = unsafe { Lapic::new(memory_allocator, 0xff) };

        // Step 4: read all of the Interrupt Source Override entries - if the IRQ source of any of them is 1 (Keyboard) use that in IOREDTBL
        let keyboard_gsi = interrupt_source_overrides
            .iter()
            .filter_map(|interrupt_source_override| {
                if interrupt_source_override.isa_source == (IsaIrq::Keyboard as u8) {
                    Some(interrupt_source_override.global_system_interrupt)
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(ioapic.global_system_interrupt_base + (IsaIrq::Keyboard as u32)); // An educated guess is that it is connected to the IOAPIC pin corresponding to its usual PIC pin

        if keyboard_gsi < ioapic.global_system_interrupt_base {
            panic!("No IOAPIC connected to keyboard");
        }

        let gsi_base = ioapic.global_system_interrupt_base;

        // Step 5: Configure the IOREDTBL entry in registers 0x12 and 0x13 (unless you need to use a different one, per the above step)
        let mut ioapic = IoApic::new(memory_allocator, ioapic);
        ioapic.set_ioredtbl((keyboard_gsi - gsi_base) as u8, 0x41, lapic.lapic_id());

        // Step 6: Enable the APIC by setting the 11th bit of the APIC base MSR (0x1B)
        let mut apic_base_msr = Msr::new(0x1b);
        unsafe { apic_base_msr.write(apic_base_msr.read() | (1 << 11)) };

        // Configure timer
        lapic.configure_timer(0x31, 0xffffff, TimerDivideConfig::DivideBy16);

        Apic
    }
}

struct IMCR {
    selector_port: Port<u8>,
    value_port: Port<u8>,
}

impl IMCR {
    fn new() -> Self {
        IMCR {
            selector_port: Port::new(0x22),
            value_port: Port::new(0x23),
        }
    }

    /// See: https://zygomatic.sourceforge.net/devref/group__arch__ia32__apic.html
    fn enable_symmetric_io_mode(&mut self) {
        unsafe {
            self.selector_port.write(0x70u8); // select IMCR
            self.value_port.write(0x01u8); // force NMI and INTR signals through the APIC}
        }
    }
}
