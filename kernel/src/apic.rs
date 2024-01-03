use crate::memory::frame_allocator::FRAME_ALLOCATOR;
use crate::memory::mapper::MAPPER;
use crate::memory::virtual_addresses::{IOAPIC_START, LAPIC_START};
use acpi::platform::interrupt::IoApic;
use acpi::InterruptModel;
use alloc::alloc::Global;
use core::ptr::slice_from_raw_parts_mut;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::registers::model_specific::Msr;
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub const LAPIC_ID_OFFSET: u64 = 0x20;
pub const SIVR_OFFSET: u64 = 0xf0;
pub const DESTINATION_FORMAT_OFFSET: u64 = 0xe0;
pub const TASK_PRIORITY_OFFSET: u64 = 0x80;
pub const INITIAL_COUNT_REGISTER_OFFSET: u64 = 0x380;
pub const LVT_TIMER_OFFSET: u64 = 0x320;
pub const DIVIDE_CONFIG_OFFSET: u64 = 0x3e0;

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

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

pub static GLOBAL_LAPIC: Mutex<Option<LAPIC>> = Mutex::new(None);

pub fn init(interrupt_model: &InterruptModel<Global>) {
    // The following section uses the overall steps from: https://blog.wesleyac.com/posts/ioapic-interrupts

    let (lapic_base, ioapics, interrupt_source_overrides) = match interrupt_model {
        InterruptModel::Apic(apic_info) => (
            (&apic_info).local_apic_address,
            &apic_info.io_apics,
            &apic_info.interrupt_source_overrides,
        ),
        _ => {
            panic!("interrupt model is not apic")
        }
    };

    // Both QEMU and my test system only have one IOAPIC so this is fine for now.
    let ioapic = &ioapics[0];

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
    LAPIC::init(lapic_base, 0xff);

    let mut apic = GLOBAL_LAPIC.lock();
    let apic = apic.as_mut().unwrap();

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
    let mut ioapic = IOAPIC::new(ioapic);
    ioapic.set_ioredtbl((keyboard_gsi - gsi_base) as u8, 0x41, apic.lapic_id());

    // Step 6: Enable the APIC by setting the 11th bit of the APIC base MSR (0x1B)
    let mut apic_base_msr = Msr::new(0x1b);
    unsafe { apic_base_msr.write(apic_base_msr.read() | (1 << 11)) };

    // Configure timer
    apic.configure_timer(0x30, 0xffffff, TimerDivideConfig::DivideBy16);
}

#[allow(dead_code)]
enum TimerDivideConfig {
    DivideBy2 = 0b0000,
    DivideBy4 = 0b0001,
    DivideBy8 = 0b0010,
    DivideBy16 = 0b0011,
    DivideBy32 = 0b1000,
    DivideBy64 = 0b1001,
    DivideBy128 = 0b1010,
    DivideBy1 = 0b1011,
}

pub struct LAPIC {
    mm_region: &'static mut [u32],
}

impl LAPIC {
    pub fn end_of_interrupt(&mut self) {
        self.write(0xB0, 0);
    }

    #[allow(dead_code)]
    fn lapic_id(&self) -> u8 {
        ((self.read(LAPIC_ID_OFFSET)) >> 24) as u8
    }

    fn init(base_address: u64, spurious_interrupt_vector: u8) {
        if GLOBAL_LAPIC.lock().is_some() {
            panic!("APIC must only be initialised once");
        }

        let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(PhysAddr::new(base_address));

        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(LAPIC_START as u64));

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

        let mut mutex_guard = FRAME_ALLOCATOR.lock();
        let frame_allocator = mutex_guard.as_mut().unwrap();

        let mut mutex_guard = MAPPER.lock();
        let mapper = mutex_guard.as_mut().unwrap();

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .unwrap()
                .flush();
        }

        let mm_region = slice_from_raw_parts_mut(page.start_address().as_mut_ptr(), 0x1000);
        let mm_region = unsafe { &mut *mm_region };

        let mut apic = LAPIC { mm_region };

        // https://forum.osdev.org/viewtopic.php?f=1&t=12045&hilit=APIC+init

        apic.write(SIVR_OFFSET, 0x100 | (spurious_interrupt_vector as u32)); // 0x100 sets bit 8 to enable APIC

        // set destination format register to flat mode
        apic.write(DESTINATION_FORMAT_OFFSET, 0xFFFFFFFF);

        // set task priority to accept all interrupts
        apic.write(TASK_PRIORITY_OFFSET, 0);

        *GLOBAL_LAPIC.lock() = Some(apic);
    }

    fn configure_timer(&mut self, vector: u8, timer_initial: u32, timer_divide: TimerDivideConfig) {
        // The order is important DO NOT CHANGE
        self.write(DIVIDE_CONFIG_OFFSET, timer_divide as u32);
        self.write(LVT_TIMER_OFFSET, (1 << 17) | (vector as u32));
        self.write(INITIAL_COUNT_REGISTER_OFFSET, timer_initial);
    }

    #[allow(dead_code)]
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

struct IOAPIC {
    ioregsel: &'static mut u32,
    iowin: &'static mut u32,
}

#[allow(dead_code)]
enum DestinationMode {
    Physical = 0,
    Logical = 1,
}

#[allow(dead_code)]
enum PinPolarity {
    ActiveHigh = 0,
    ActiveLow = 1,
}

impl IOAPIC {
    fn set_ioredtbl(&mut self, irq: u8, vector: u8, lapic_id: u8) {
        let low_offset = 0x10 + irq * 2;
        let high_offset = 0x10 + irq * 2 + 1;

        let ioredtbl = (self.read(low_offset) as u64) | ((self.read(high_offset) as u64) << 32);

        let delivery_mode = 0b000u8;

        let destination_mode = DestinationMode::Physical as u8;

        let pin_polarity = PinPolarity::ActiveHigh as u8;

        let ioredtbl = (ioredtbl & !0x0f0_0000_0001_efff)
            | (vector as u64)
            | (((delivery_mode & 0b111) as u64) << 8)
            | (((destination_mode & 0b1) as u64) << 11)
            | (((pin_polarity & 0b1) as u64) << 13)
            | (((lapic_id & 0xf) as u64) << 56);

        self.write(low_offset, ioredtbl as u32);
        self.write(high_offset, (ioredtbl >> 32) as u32)
    }

    fn read(&mut self, offset: u8) -> u32 {
        *self.ioregsel = offset as u32;
        *self.iowin
    }

    fn write(&mut self, offset: u8, value: u32) {
        *self.ioregsel = offset as u32;
        *self.iowin = value;
    }

    fn new(io_apic: &IoApic) -> Self {
        let base_address = io_apic.address;

        let frame: PhysFrame<Size4KiB> =
            PhysFrame::containing_address(PhysAddr::new(base_address as u64));

        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(IOAPIC_START as u64));

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

        let mut mutex_guard = FRAME_ALLOCATOR.lock();
        let frame_allocator = mutex_guard.as_mut().unwrap();

        let mut mutex_guard = MAPPER.lock();
        let mapper = mutex_guard.as_mut().unwrap();

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .unwrap()
                .flush();
        }

        let ioregsel: *mut u32 = VirtAddr::new(IOAPIC_START as u64).as_mut_ptr();
        let ioregsel = unsafe { &mut *ioregsel };

        let iowin: *mut u32 = VirtAddr::new((IOAPIC_START as u64) + 0x10).as_mut_ptr();
        let iowin = unsafe { &mut *iowin };

        IOAPIC { ioregsel, iowin }
    }
}
