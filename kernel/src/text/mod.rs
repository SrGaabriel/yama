pub mod color;
mod fonts;

use core::fmt;
use bootloader_api::info::{FrameBuffer, PixelFormat};
use color::ColorCode;
use crate::text::color::Color;

pub struct TextWriter {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut [u8],
    info: bootloader_api::info::FrameBufferInfo,
    font: &'static [u8]
}

impl TextWriter {
    pub fn new_framebuffer_writer(frame_buffer: &'static mut FrameBuffer, color_code: ColorCode) -> TextWriter {
        let info = frame_buffer.info();
        let mut writer = TextWriter {
            column_position: 0,
            row_position: 0,
            color_code,
            buffer: frame_buffer.buffer_mut(),
            info,
            font: fonts::FONT8X8_BASIC_FLATTENED.as_slice(),
        };
        writer.clear_screen();
        writer
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel_offset = y * (self.info.stride * self.info.bytes_per_pixel) +
            x * self.info.bytes_per_pixel;

        if pixel_offset + self.info.bytes_per_pixel > self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[pixel_offset] = color.0;
                self.buffer[pixel_offset + 1] = color.1;
                self.buffer[pixel_offset + 2] = color.2;
            }
            PixelFormat::Bgr => {
                self.buffer[pixel_offset] = color.2;
                self.buffer[pixel_offset + 1] = color.1;
                self.buffer[pixel_offset + 2] = color.0;
            }
            PixelFormat::U8 => {
                let gray = ((color.0 as f32 * 0.299 +
                    color.1 as f32 * 0.587 +
                    color.2 as f32 * 0.114) as u8);
                self.buffer[pixel_offset] = gray;
            }
            PixelFormat::Unknown { red_position, green_position, blue_position } => {
                let mut pixel: u32 = 0;
                pixel |= (color.0 as u32) << red_position;
                pixel |= (color.1 as u32) << green_position;
                pixel |= (color.2 as u32) << blue_position;

                for i in 0..self.info.bytes_per_pixel {
                    self.buffer[pixel_offset + i] = (pixel >> (i * 8)) as u8;
                }
            }
            _ => unreachable!("Unsupported pixel format"),
        }
    }

    fn scroll_up(&mut self) {
        let bytes_per_line = self.info.stride * self.info.bytes_per_pixel;
        let line_count = 8;

        let source_start = line_count * bytes_per_line;
        let copy_size = (self.info.height - line_count) * bytes_per_line;

        if source_start + copy_size <= self.buffer.len() {
            self.buffer.copy_within(source_start..source_start + copy_size, 0);
        }

        let clear_start = (self.info.height - line_count) * bytes_per_line;
        if clear_start < self.buffer.len() {
            let clear_end = core::cmp::min(
                clear_start + (line_count * bytes_per_line),
                self.buffer.len()
            );
            self.buffer[clear_start..clear_end].fill(0);
        }

        self.row_position -= line_count;
    }

    fn clear_screen(&mut self) {
        let bytes_per_line = self.info.stride * self.info.bytes_per_pixel;
        let total_size = self.info.height * bytes_per_line;

        let clear_size = core::cmp::min(total_size, self.buffer.len());
        self.buffer[0..clear_size].fill(0);

        self.column_position = 0;
        self.row_position = 0;
    }

    fn color_to_rgb(&self, color: ColorCode) -> (u8, u8, u8) {
        match Color::from_u8(color.0) {
            Color::Black => (0, 0, 0),
            Color::Blue => (0, 0, 255),
            Color::Green => (0, 255, 0),
            Color::Cyan => (0, 255, 255),
            Color::Red => (255, 0, 0),
            Color::Magenta => (255, 0, 255),
            Color::Brown => (165, 42, 42),
            Color::LightGray => (211, 211, 211),
            Color::DarkGray => (169, 169, 169),
            Color::LightBlue => (173, 216, 230),
            Color::LightGreen => (144, 238, 144),
            Color::LightCyan => (224, 255, 255),
            Color::LightRed => (255, 182, 193),
            Color::Pink => (255, 192, 203),
            Color::Yellow => (255, 255, 0),
            Color::White => (255, 255, 255),
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= self.info.width - 8 {
                    self.new_line();
                }

                let char_idx = byte as usize * 8;
                let font_char = &self.font[char_idx..char_idx + 8];
                let color = self.color_to_rgb(self.color_code);

                for (y, &row) in font_char.iter().enumerate() {
                    for x in 0..8 {
                        if (row >> (7 - x)) & 1 == 1 {
                            self.write_pixel(
                                self.column_position + x,
                                self.row_position + y,
                                color
                            );
                        }
                    }
                }

                self.column_position += 8;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        self.row_position += 8;
        self.column_position = 0;

        if self.row_position >= self.info.height - 8 {
            self.scroll_up();
        }
    }

    fn clear_last_line(&mut self) {
        let start = (self.info.height - 8) * self.info.stride;
        let end = self.info.height * self.info.stride;

        if end <= self.buffer.len() {
            self.buffer[start..end].fill(0);
        }
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use uart_16550::SerialPort;

pub struct DualWriter {
    framebuffer: TextWriter,
    serial: SerialPort,
}

impl DualWriter {
    pub fn new(framebuffer: TextWriter, serial: SerialPort) -> Self {
        DualWriter {
            framebuffer,
            serial,
        }
    }

    pub fn write_string(&mut self, s: &str) {
        self.framebuffer.write_string(s);
        for byte in s.bytes() {
            self.serial.send(byte);
        }
    }
}

impl fmt::Write for DualWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn init_writers(
    frame_buffer: &'static mut FrameBuffer,
    color_code: ColorCode
) -> DualWriter {
    let framebuffer = TextWriter::new_framebuffer_writer(frame_buffer, color_code);

    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();

    DualWriter::new(framebuffer, serial)
}
