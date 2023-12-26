// lib.rs是一个单独的编译单元, 需要单独指定一些属性
// 此外, 还需要将main.rs中的一些函数迁移到该目录下
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

// 在lib.rs中声明我们实现的两个简易的模块
// 这样才能使外部库使用, main.rs则无需使用单独声明
pub mod serial;
pub mod vga_buffer;

pub trait Testable {
    fn run(&self) -> ();
}

// 实现上面定义的trait
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self(); // 该调用时Fn() trait独有, 将会调用需要测试的函数
        serial_println!("[OK]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
        // test();
    }
    // 进行退出检测
    // 对于所有执行的检测, cargo test会将所有非0的错误码都视为测试失败
    // 通过bootimage设置, 将成功的测试结果映射到退出码0
    exit_qemu(QemuExitCode::Success);
}

// 重构panic, 便于后续复用
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// 测试环境的入口
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// 该函数在0xf4处创建了一个新的端口, 参数也通过该端口传入
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

