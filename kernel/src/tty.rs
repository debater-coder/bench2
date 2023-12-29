use core::fmt;
use core::fmt::Write;
use bootloader_api::info::FrameBufferInfo;
use noto_sans_mono_bitmap::{FontWeight, get_raster, RasterHeight, get_raster_width};

pub struct Writer<'a> {
    column: usize,
    raw_framebuffer: &'a mut [u8],
    framebuffer_info: FrameBufferInfo
}

pub fn print_something(raw_framebuffer: &mut [u8], framebuffer_info: FrameBufferInfo) {
    let mut writer = Writer {
        column: 0,
        raw_framebuffer,
        framebuffer_info,
    };

    writer.write_string("Hello!\n");
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}

impl Writer<'_> {
    fn write_pixel(&mut self, x: usize, y: usize, pixel: u8) {
        self.raw_framebuffer[y * self.framebuffer_info.stride * 4 + x * 4] = pixel;
        self.raw_framebuffer[y * self.framebuffer_info.stride * 4 + x * 4 + 1] = pixel;
        self.raw_framebuffer[y * self.framebuffer_info.stride * 4 + x * 4 + 2] = pixel;
    }

    pub fn write_char(&mut self, character: char) {
        match character {
            '\n' => self.new_line(),
            character => {
                if self.column >= self.framebuffer_info.width {
                    self.new_line();
                }

                let y = self.framebuffer_info.height - 16;
                let x = self.column;

                let char_raster = get_raster(character, FontWeight::Regular, RasterHeight::Size16).unwrap_or(get_raster('?', FontWeight::Regular, RasterHeight::Size16).expect("fallback char not supported"));

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
        for row in 16..self.framebuffer_info.height {
            for col in 0..self.framebuffer_info.width {
                let pixel = self.raw_framebuffer[row * self.framebuffer_info.stride * 4 + col * 4];
                self.write_pixel(col, row - 16, pixel);
            }
        }
        for row in (self.framebuffer_info.height - 16)..self.framebuffer_info.height {
            for col in 0..self.framebuffer_info.width {
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