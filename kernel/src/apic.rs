use crate::apic;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub fn init() {
    unsafe {
        // This remaps it so that when we have a spurious interrupt it doesn't mess us up
        apic::PICS.lock().initialize();

        PICS.lock().disable();
    }

    IMCR::enable_symmetric_io_mode();
}

struct IMCR;

impl IMCR {
    /// See: https://zygomatic.sourceforge.net/devref/group__arch__ia32__apic.html
    fn enable_symmetric_io_mode() {
        unsafe { Port::new(0x22).write(0x70u8) }; // select IMCR
        unsafe { Port::new(0x23).write(0x01u8) }; // force NMI and INTR signals through the APIC
    }
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
