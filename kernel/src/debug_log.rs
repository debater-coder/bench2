use crate::gop_buffer::WRITER;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub struct DebugPort {
    port: Port<u8>,
}

lazy_static! {
    pub static ref DEBUG_PORT: Mutex<DebugPort> = Mutex::new(DebugPort {
        port: Port::new(0xe9)
    });
}

impl DebugPort {
    pub fn write_byte(&mut self, character: u8) {
        unsafe { self.port.write(character) }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }
}

impl fmt::Write for DebugPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::debug_log::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    DEBUG_PORT.lock().write_fmt(args).unwrap();

    let mut writer = WRITER.lock();
    if let Some(writer) = writer.as_mut() {
        writer.write_fmt(args).unwrap();
    }
}
