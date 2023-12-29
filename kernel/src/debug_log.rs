use core::fmt;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;
use spin::Mutex;

pub struct DebugPort {
    port: Port<u8>
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