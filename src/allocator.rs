use core::{alloc::GlobalAlloc, ptr::null_mut};

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;

#[global_allocator]
// 使用现有的crate来实现分配器
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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
        // 使用x86_64crate提供的FrameAllocator分配内存帧,
        // ok_or将分配的结果转换为Result<Frame, MapToError>类型
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        // 判断可用和可写的标志
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        // 在页表中创建新的映射项, 
        // 将页面信息, 页框, 标志, 分配器传进去, 返回更改了的页面, 并进行刷新
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    // 初始化分配器
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Dummy;

// 创建了一个虚假的分配器, 不具有实际意义
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}
