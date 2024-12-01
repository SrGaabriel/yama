pub struct GpuDevice {
    pub mmio_base: *mut u8,
    pub framebuffer: *mut u8
}

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub format: u32,
    pub buffer: *mut u8
}