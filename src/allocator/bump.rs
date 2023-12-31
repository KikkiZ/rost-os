use core::{alloc::GlobalAlloc, ptr};

use linked_list_allocator::align_up;

use super::Locked;

// bump分配器
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    // 创建一个空的分配器
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    // 初始化分配器
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let alloc_start = align_up(allocator.next, layout.size());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        // 检测分配是否超出堆区大小
        if alloc_end > allocator.heap_end {
            ptr::null_mut()
        } else {
            allocator.next = alloc_end;
            allocator.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut allocator = self.lock();

        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}
