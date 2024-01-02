use core::{alloc::GlobalAlloc, ptr::null_mut};

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100KiB

// 初始化堆
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        // 计算堆的起始和结束地址
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;

        // 获取堆的起始和结束页
        let heap_start_page: Page<Size4KiB> = Page::containing_address(heap_start);
        let heap_end_page: Page<Size4KiB> = Page::containing_address(heap_end);

        // 返回堆的范围
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    // 初始化分配器
    unsafe {
        crate::ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}

pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

// fn align_up(addr: usize, align: usize) -> usize {
//     // 该段代码实现了基础功能, 但下面的实现更快速
//     // let remainder = addr % align;
//     // if remainder == 0 {
//     //     addr
//     // } else {
//     //     addr - remainder + align
//     // }

//     (addr + align - 1) & !(align - 1)
// }
