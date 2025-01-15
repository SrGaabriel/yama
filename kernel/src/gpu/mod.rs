pub mod scan;
mod debug;
mod mmio;

use core::fmt::Display;
use bootloader_api::BootInfo;
use crate::gpu::scan::KnownVendor;
use crate::mem::Vec;

pub struct PciDevice {
    vendor_id: u16,
    device_id: u16,
    base_address: usize
}

impl PciDevice {
    pub fn new(vendor_id: u16, device_id: u16, base_address: usize) -> Self {
        PciDevice {
            vendor_id,
            device_id,
            base_address
        }
    }

    pub fn known_vendor(&self) -> Option<KnownVendor> {
        match self.vendor_id {
            0x10DE => Some(KnownVendor::Nvidia),
            0x1002 => Some(KnownVendor::Amd),
            0x8086 => Some(KnownVendor::Intel),
            0x1234 => Some(KnownVendor::Dummy),
            _ => None
        }
    }
}

pub struct GpuDriver {
    pub mmio_base: *mut u8,
    pub framebuffer: *mut u8
}

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub buffer: Vec<u8>
}

const MMIO_SIZE: usize = 0x1000000;

impl GpuDriver {
    pub fn from_bootinfo(boot_info: &mut BootInfo) -> Option<Self> {
        let pci_device = scan::detect_gpu()?;
        let mmio_base = mmio::map_mmio_space(pci_device.base_address, MMIO_SIZE);
        let framebuffer = unsafe { boot_info.framebuffer.as_mut() }?;
        None
    }
}