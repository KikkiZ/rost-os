#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

// panic_handler在test模式下与非test模式下都需要存在
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    println!("Hello RUST!");

    rust_os::init(); // 初始化
    x86_64::instructions::interrupts::int3();

    // 递归调用导致栈溢出, 造成page fault异常,
    // 调用对应的处理程序, 但是栈溢出, 无法入栈,
    // 再次page falut, 反复三次导致triple fault异常,
    // 对于这种异常, 注册错误处理函数难以处理所有情况
    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    // 直接操作了一个无效的内存地址
    // 这会造成一个page fault异常, 系统在idt中没有找到对应的处理函数
    // 然后将会抛出double fault异常, 系统将会无限重启
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}
