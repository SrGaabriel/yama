const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    frame: usize,
    flags: u32
}

pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_SIZE]
}

impl PageTable {
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry {
                frame: 0,
                flags: 0
            }; PAGE_TABLE_SIZE]
        }
    }

    pub fn map(&mut self, virtual_addr: usize, physical_addr: usize, flags: u32) {
        let index = virtual_addr / PAGE_SIZE;
        self.entries[index] = PageTableEntry {
            frame: physical_addr / PAGE_SIZE,
            flags
        };
    }
}

pub fn map_mmio_space(base: usize, size: usize) -> usize {
    let mut page_table = PageTable::new();

    let mut addr = base;
    let end = base + size;

    while addr < end {
        let physical_addr = addr;
        let flags = 0x03;
        page_table.map(addr, physical_addr, flags);
        addr += PAGE_SIZE;
    }

    base
}