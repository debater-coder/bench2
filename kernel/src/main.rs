#![no_std]
#![no_main]

use bootloader_api::config::Mapping;
use bootloader_api::BootloaderConfig;
use core::panic::PanicInfo;
use pc_keyboard::DecodedKey::Unicode;

use kernel::io::keyboard::Keyboard;
use kernel::{init, print, println};

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(0x20000000000));
    config
};

bootloader_api::entry_point!(kernel_early, config = &BOOTLOADER_CONFIG);

fn kernel_early(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    unsafe { init(boot_info) };

    print!("> ");

    let mut keyboard = Keyboard::new();

    loop {
        if let Some(Unicode(key)) = keyboard.poll_next() {
            print!("{}", key);
        }
    }
}
