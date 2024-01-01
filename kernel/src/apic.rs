use core::ptr::slice_from_raw_parts_mut;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::VirtAddr;

pub const SIVR_OFFSET: u64 = 0xf0;

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// see: https://blog.wesleyac.com/posts/ioapic-interrupts
pub fn init(physical_memory_offset: VirtAddr) {
    // Step 1: Disable the PIC.
    unsafe {
        // This remaps it so that when we have a spurious interrupt it doesn't mess us up
        PICS.lock().initialize();

        PICS.lock().disable();
    }

    // Step 2: Set IMCR (IMCR is optional if PIC mode is not implemented so it is uncommon on modern systems)
    let mut imcr = IMCR::new();
    imcr.enable_symmetric_io_mode();

    // Step 3: Configure the "Spurious Interrupt Vector Register" of the Local APIC to 0xFF
    let mm_region: &mut [u32] = unsafe {
        &mut *slice_from_raw_parts_mut(
            VirtAddr::new(0xFEE0_0000 + physical_memory_offset).as_mut_ptr(),
            0x1000,
        )
    };

    let mut apic = APIC { mm_region };

    apic.write(SIVR_OFFSET, 0xff);
}

struct APIC {
    mm_region: &'static mut [u32],
}

impl APIC {
    fn read(&self, offset: u64) -> u32 {
        self.mm_region[offset as usize / 4]
    }
    fn write(&mut self, offset: u64, val: u32) {
        self.mm_region[offset as usize / 4] = val;
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

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
