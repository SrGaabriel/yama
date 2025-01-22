#![no_std]
#![no_main]
#![feature(const_slice_flatten)]

extern crate alloc;

use alloc::string::ToString;
use core::arch::asm;
use core::fmt::Write;
use bootloader_api::{entry_point, BootInfo};
use crate::gpu::scan::detect_gpu;

mod panic;
mod text;
mod gpu;
mod mem;
mod clock;

use crate::text::color::{Color, ColorCode};
use crate::text::{init_writers, DualWriter, TextWriter};

entry_point!(kernel_main);

pub struct KernelState {
    dual_writer: DualWriter,
}

impl KernelState {
    pub fn new(framebuffer: &'static mut bootloader_api::info::FrameBuffer) -> Self {
        let dual_writer = init_writers(framebuffer, ColorCode::new(Color::White, Color::Black));
        Self {
            dual_writer
        }
    }

    pub fn write_fmt(&mut self, args: core::fmt::Arguments) -> core::fmt::Result {
        self.dual_writer.write_fmt(args)
    }
}

static KERNEL: spin::Once<spin::Mutex<KernelState>> = spin::Once::new();

pub fn kernel() -> &'static spin::Mutex<KernelState> {
    KERNEL.get().expect("Kernel not initialized")
}

pub fn init(boot_info: &'static mut BootInfo) {
    let framebuffer = boot_info.framebuffer.as_mut()
        .expect("Framebuffer required");

    KERNEL.call_once(|| spin::Mutex::new(KernelState::new(framebuffer)));
}

pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);
    println!("Hello, world!");

    let gpu = detect_gpu();
    println!("GPU: {:?}", gpu);
    println!("TSC: {}", clock::read_tsc());

    let is_vm = gpu.is_some_and(|gpu| gpu.known_vendor() == Some(gpu::scan::KnownVendor::Virtual));
    if is_vm {
        println!("Running in a virtual machine");
    }

    loop {}
}

pub fn exit(code: u32) -> ! {
    unsafe {
        asm!("ud2", options(nomem, nostack));
    }
    loop {}
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        let kernel = $crate::kernel();
        let mut kernel = kernel.lock();
        let _ = kernel.write_fmt(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", format_args!($($arg)*));
    }};
}