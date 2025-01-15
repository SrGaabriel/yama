use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{null_mut, NonNull};

const MAX_ORDER: usize = 16;
const MEMORY_SIZE: usize = 1 << MAX_ORDER;

pub struct BuddyAllocator {
    memory: UnsafeCell<[u8; MEMORY_SIZE]>,
    free_lists: UnsafeCell<[Option<NonNull<u8>>; MAX_ORDER + 1]>,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        BuddyAllocator {
            memory: UnsafeCell::new([0; MEMORY_SIZE]),
            free_lists: UnsafeCell::new([None; MAX_ORDER + 1]),
        }
    }

    fn init(&self) {
        unsafe {
            let memory_start = self.memory.get() as *mut u8;
            let free_lists = self.free_lists.get();
            (*free_lists)[MAX_ORDER] = NonNull::new(memory_start);
        }
    }

    fn calculate_order(size: usize) -> usize {
        size.next_power_of_two().trailing_zeros() as usize
    }

    unsafe fn split_block(&self, addr: *mut u8, current_order: usize) -> (*mut u8, *mut u8) {
        let block_size = 1 << current_order;
        let half_size = block_size >> 1;

        let left = addr;
        let right = (addr as usize + half_size) as *mut u8;

        let free_lists = self.free_lists.get();

        (*free_lists)[current_order] = None;

        (*free_lists)[current_order - 1] = NonNull::new(left);
        (*free_lists)[current_order - 1] = NonNull::new(right);

        (left, right)
    }
}

unsafe impl Sync for BuddyAllocator {}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.free_lists.get().read()[MAX_ORDER].is_none() {
            self.init();
        }

        let size = layout.size().next_power_of_two().max(layout.align());
        let order = Self::calculate_order(size);

        let mut current_order = order;
        let mut block = null_mut();

        while current_order <= MAX_ORDER {
            let free_lists = self.free_lists.get();

            if let Some(free_block) = (*free_lists)[current_order] {
                block = free_block.as_ptr();
                (*free_lists)[current_order] = None;
                break;
            }

            current_order += 1;
        }

        if block.is_null() {
            return null_mut();
        }

        while current_order > order {
            let (left, _) = self.split_block(block, current_order);
            current_order -= 1;
            block = left;
        }

        block
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let size = layout.size().next_power_of_two().max(layout.align());
        let order = Self::calculate_order(size);

        let free_lists = self.free_lists.get();
        (*free_lists)[order] = NonNull::new(ptr);
    }
}

#[global_allocator]
static ALLOCATOR: BuddyAllocator = BuddyAllocator::new();
