use core::{alloc::GlobalAlloc, ptr};

use linked_list_allocator::align_up;

use super::Locked;

// bump分配器
pub struct BumpAllocator {
    heap_start: usize,  // 堆的起始地址
    heap_end: usize,    // 堆的结束地址
    next: usize,        // 记录未分配的堆的起始地址
    allocations: usize, // 记录堆记录分配数的简单计数器, 在释放最后一个分配后重置为0
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

// 实现GlobalAlloc trait
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    // 不可以在self的引用上修改, 因为没有声明是可变的,
    // 而这个trait是预先定义好的, 我们无法修改.
    // 我们可以通过spin::Mutex自旋锁进行包装, 使用同步内部可变性,
    // 安全的将&self转为&mut self引用
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // 从不可变的引用中获取可变的分配器
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

        // 简单的回收堆内存, 只有全部的内存都收回了才会从头开始
        // 这是一个非常低效的实现, 对内存的利用率非常低
        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}
