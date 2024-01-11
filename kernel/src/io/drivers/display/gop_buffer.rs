use crate::io::framebuffer::FRAMEBUFFER;
use core::fmt;
use lazy_static::lazy_static;
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use spin::{Mutex, MutexGuard};

lazy_static! {
    pub static ref WRITER: Mutex<Option<Writer<'static>>> = Mutex::new(None);
}

pub struct Writer<'a> {
    column: usize,
    raw_framebuffer_lock: MutexGuard<'a, &'static mut [u8]>,
}

impl Writer<'_> {
    pub unsafe fn init() {
        *WRITER.lock() = Some(Writer {
            column: 0,
            raw_framebuffer_lock: FRAMEBUFFER.try_get().unwrap().raw_framebuffer.lock(),
        });
    }

    fn write_pixel(&mut self, x: usize, y: usize, pixel: u8) {
        let stride = FRAMEBUFFER.get().unwrap().framebuffer_info.stride;
        let raw_framebuffer = &mut self.raw_framebuffer_lock;

        raw_framebuffer[y * stride * 4 + x * 4] = pixel;
        raw_framebuffer[y * stride * 4 + x * 4 + 1] = pixel;
        raw_framebuffer[y * stride * 4 + x * 4 + 2] = pixel;
    }

    pub fn write_char(&mut self, character: char) {
        let framebuffer_info = &FRAMEBUFFER.get().unwrap().framebuffer_info;

        match character {
            '\n' => self.new_line(),
            character => {
                if self.column + 16 >= framebuffer_info.width {
                    self.new_line();
                }

                let y = framebuffer_info.height - 16;
                let x = self.column;

                let char_raster = get_raster(character, FontWeight::Regular, RasterHeight::Size16)
                    .unwrap_or(
                        get_raster('?', FontWeight::Regular, RasterHeight::Size16)
                            .expect("fallback char not supported"),
                    );

                for (row_i, row) in char_raster.raster().iter().enumerate() {
                    for (col_i, pixel) in row.iter().enumerate() {
                        self.write_pixel(x + col_i, y + row_i, *pixel)
                    }
                }

                self.column += get_raster_width(FontWeight::Regular, RasterHeight::Size16);
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for char in s.chars() {
            self.write_char(char);
        }
    }
    fn new_line(&mut self) {
        let framebuffer_info = &FRAMEBUFFER.get().unwrap().framebuffer_info;

        for row in 16..framebuffer_info.height {
            for col in 0..framebuffer_info.width {
                let raw_framebuffer = &mut self.raw_framebuffer_lock;
                let pixel = raw_framebuffer[row * framebuffer_info.stride * 4 + col * 4];

                self.write_pixel(col, row - 16, pixel);
            }
        }
        for row in (framebuffer_info.height - 16)..framebuffer_info.height {
            for col in 0..framebuffer_info.width {
                self.write_pixel(col, row, 0);
            }
        }

        self.column = 0;
    }
}

impl fmt::Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
