#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    println!("Hello RUST!");

    use rust_os::warn;
    warn!("test warn");

    rust_os::init(); // 初始化

    // 测试页表错误
    let ptr = 0xdeadbeaf as *mut u8;
    unsafe { *ptr = 42; }
    // use x86_64::registers::control::Cr3;
    // let (level_4_page_table, _) = Cr3::read();
    // println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rust_os::hlt_loop();
}
