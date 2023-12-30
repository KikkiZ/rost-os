use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

// 初始化
// physical_memory_offset 是地址的偏移量, 用于计算虚拟地址
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// 返回一个对活动的4级表的可变引用
// 这个函数是不安全的, 且只能被调用一次, 以避免意外的问题
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // x86_64::registers::control下包含了很多结构体
    // Cr0 -> 修改CPU基本操作的各种控制标志
    // Cr2 -> 包含页面错误的线性地址
    // Cr3 -> 包含最高级别页表的物理地址
    // Cr4 -> 包含启用架构拓展的各种标志, 以及对特定处理器功能支持

    // Cr3.read() 从寄存器读取p4表地址, 并返回PhysFrame(物理帧)和Cr3Flags(页表的各种设置选项)
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address(); // 获取物理帧的起始地址
    let virt = physical_memory_offset + phys.as_u64(); // 通过偏移量和物理地址设置虚拟地址
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr(); // 将虚拟地址转换为原始指针
    // 原始指针是没有安全性或生命保证的指针, 取消引用原始指针是一种不安全的操作
    // 原始指针可以用于重新借用或转换为引用, 通常用于编写性能相关型或底层函数, 不鼓励使用

    &mut *page_table_ptr // unsafe
}

pub fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    _translate_addr(addr, physical_memory_offset)
}

fn _translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    // 从Cr3寄存器中读取4级表
    let (level_4_table_frame, _) = Cr3::read();

    let table_index = [
        addr.p4_index(), // 1级页表索引
        addr.p3_index(), // 2级页表索引
        addr.p2_index(), // 3级页表索引
        addr.p1_index(), // 4级页表索引
    ];
    let mut frame = level_4_table_frame; // 4级索引的物理帧

    // 遍历多级页表
    for &index in &table_index {
        // 将4级页表的物理地址转换为虚拟地址
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr(); // 将虚拟地址的原始指针转为页表指针
        let table = unsafe { &*table_ptr }; // 转为页表结构体

        // 读取页表条目并更新
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame)             => frame,
            Err(FrameError::FrameNotPresent) => return None, // 该条目未分配 
            Err(FrameError::HugeFrame)       => panic!("huge pages not supported"), //该条目已分配, 但帧的大小过大, 无法返回
        };
    }

    // 计算物理地址
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

// 为给定的页面创建一个实例映射到框架`0xb8000`。
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // 这并不安全，我们这样做只是为了测试。
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

// 一个FrameAllocator，从bootloader的内存地址中返回可用的物理内存映射
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    // 从传递的内存映射表中创建一个FrameAllocator,
    // 这个函数是不安全的，因为使用者必须保证传递的内存map是有效的,
    // 主要的要求是, 所有在其中被标记为"可用"的帧都是真正未使用的
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // 返回内存映射中指定的可用框架的迭代器
    fn unable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter(); // 获取MemoryMap的迭代器, 用于遍历内存映射表
        // 使用迭代器附带的过滤方法, 使用lambda表达式, 遍历每一个物理内存区域(MemoryRegion)
        // 通过其中的一个类型变量(MemoryRegionType)判断内存区域的类型 -> 是否可用
        // 最后返回所有没有没有使用的内存区域
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // 将每个区域映射到其地址范围, '..'操作符将会获得一个范围
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // 转化为一个帧起始地址的迭代器, 将范围(range)对象的步长修改为4096(4kb)大小
        // 此时的frame_addresses中所包含的数据是每一个内存页的起始地址
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 遍历每一个可能的内存页的起始地址, 通过物理地址(PhysAddr)创建指定位置的物理帧
        // 并将所有创建的物理帧返回
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    // 该方法目前会在每次分配内存时调用一次unable_frame, 
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.unable_frames().nth(self.next); // 获取所有未分配的物理帧, 并分配第(next)帧
        self.next += 1; // 计数器加1
        frame
    }
}
