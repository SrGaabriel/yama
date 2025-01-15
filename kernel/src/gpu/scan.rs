use core::arch::asm;
use crate::gpu::PciDevice;

#[derive(Debug)]
pub enum KnownVendor {
    Nvidia = 0x10DE,
    Amd = 0x1002,
    Intel = 0x8086,
    Dummy = 0x1234
}

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

fn outl(port: u16, value: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") port, in("eax") value);
    }
}

fn inl(port: u16) -> u32 {
    let result: u32;
    unsafe {
        asm!("in eax, dx", out("eax") result, in("dx") port);
    }
    result
}

fn pci_config_address(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    (1 << 31) |
        ((bus as u32) << 16) |
        ((device as u32) << 11) |
        ((func as u32) << 8) |
        (offset as u32 & 0xFC)
}

fn read_pci_register(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    let address = pci_config_address(bus, device, func, offset);
    outl(PCI_CONFIG_ADDRESS, address);
    inl(PCI_CONFIG_DATA)
}

pub fn detect_gpu() -> Option<PciDevice> {
    for bus in 0..=255 {
        for device in 0..32 {
            for func in 0..8 {
                let vendor_device = read_pci_register(bus, device, func, 0x00);
                let vendor_id = (vendor_device & 0xFFFF) as u16;
                let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;

                if vendor_id == 0x0000 || vendor_id == 0xFFFF {
                    continue;
                }

                let class_code = read_pci_register(bus, device, func, 0x08);
                let base_class = ((class_code >> 24) & 0xFF) as u8;
                let subclass = ((class_code >> 16) & 0xFF) as u8;

                if base_class == 0x03 && subclass == 0x00 {
                    let bar0 = read_pci_register(bus, device, func, 0x10);
                    return Some(PciDevice::new(vendor_id, device_id, bar0 as usize));
                }
            }
        }
    }
    None
}
