// 定义集成测试环境, 
// 集成测试都是单独的可执行文件, 需要重新实现一些属性
// 该环境下无需使用cfg(test)属性, 因为集成测试环境的二进制
// 文件仅在测试模式下编译
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

fn test_runner(_test: &[&dyn Fn()]) {
    unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 调用lib.rs中定义的函数
    rust_os::test_panic_handler(info)
}

use rust_os::println;

#[test_case]
fn test_println() {
    println!("test_println output");
}
