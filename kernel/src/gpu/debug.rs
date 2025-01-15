use core::fmt::Debug;
use crate::gpu::PciDevice;

impl Debug for PciDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PciDevice {{ vendor_id: {:#X}, device_id: {:#X}, base_address: {:#X}, known_vendor: {:?} }}", self.vendor_id, self.device_id, self.base_address, self.known_vendor())
    }
}