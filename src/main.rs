#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::{memory::BootInfoFrameAllocator, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

// entry_point宏提供了一种声明系统入口的方法, 便于我们重写
// 系统的入口, 而不用exter "C"来作为入口
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    println!("Hello RUST!");

    use rust_os::warn;
    warn!("test warn");

    rust_os::init(); // 初始化

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = memory::EmptyFrameAllocator;

    // 映射未使用的页
    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // 通过新的映射将字符串 `New!`  写到屏幕上。
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    // let phys_men_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mapper = unsafe { memory::init(phys_men_offset) };
    // let addresses = [
    //     0xb8000,
    //     0x201008,
    //     0x0100_0020_1a10,
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);

    //     println!("{:?} -> {:?}", virt, phys);
    // }

    // 遍历查看已使用的四级表
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    // let phys = entry.frame().unwrap().start_address();
    // let virt = phys.as_u64() + boot_info.physical_memory_offset;
    // let ptr = VirtAddr::new(virt).as_mut_ptr();
    // let l3_table: &PageTable = unsafe { &*ptr };

    // for (i, entry) in l3_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("  L3 Entry {}: {:?}", i, entry);
    //     }
    // }
    //     }
    // }

    println!("It did not crash!");
    rust_os::hlt_loop();
}

// #[no_mangle]
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
//     println!("Hello World{}", "!");
//     println!("Hello RUST!");

//     use rust_os::warn;
//     warn!("test warn");

//     rust_os::init(); // 初始化

// 测试页表错误
// let ptr = 0xdeadbeaf as *mut u8;
// unsafe { *ptr = 42; }
// use x86_64::registers::control::Cr3;
// let (level_4_page_table, _) = Cr3::read();
// println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

//     #[cfg(test)]
//     test_main();

//     println!("It did not crash!");
//     rust_os::hlt_loop();
// }
