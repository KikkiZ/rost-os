// 缺少test库, 导致无法使用cargo test
#![no_std] // 告诉编译器不使用标准库, 只使用用核心库
#![no_main]
// 删除main函数, 告诉编译器无需初始化
// 这几行需要放在代码开头, 否则会报错
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
// 将test_runner重命名为指定名字
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;

// use crate::vga_buffer::Writer;
use core::panic::PanicInfo;

// 在测试条件下将不执行该panic, 转而执行下一个panic
#[cfg(not(test))]
// 当移除标准库之后, 核心库中不包含panic宏的具体实现
// 因此需要手动实现该模块
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[FAIL]");
    serial_println!("Error Info: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
// 使用了新定义的Testable trait, 并将函数调用更改
// fn test_runner(tests: &[&dyn Fn()]) {
fn test_runner(tests: &[&dyn Testable]) {
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

// static HELLO: &[u8] = b"Hello World!";  // 预定义字节字符串

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // vga字符模式是打印字符到屏幕的一种简单方式
    // let vga_buffer = 0xb8000 as *mut u8;  // 将整数转换为指针

    // 迭代输出字符串
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     // unsafe会破坏内存安全, 后续将会封装, 防止不安全的问题
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    // 当我们定义了一个静态变量用于输出后, 可以直接用它来实现输出
    // Writer::print_sth();

    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again,").unwrap();
    // write!(vga_buffer::WRITER.lock(), "today is {}-{}", 12, 25).unwrap();

    // 实现了print宏和println宏
    println!("Hello World{}", "!");
    println!("Hello RUST!");

    // 该测试只在上下文环境为测试(test)时调用该函数
    // 并将递归的调用需要测试的函数
    #[cfg(test)]
    test_main();

    // panic!("This is a test for Panic Msg.");

    loop {}
}

// fn main() {
// 当我们不适用标准库之后, 输出宏将无法使用
// println!("Hello, world!");
// }

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
    // 实现了自动打印日志, 无需手动打印
    // serial_println!("[OK]");
}

// 建立一个用于测试的自动日志输出trait
pub trait Testable {
    fn run(&self) -> ();
}

// 实现上面定义的trait
impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self(); // 该调用时Fn() trait独有, 将会调用需要测试的函数
        serial_println!("[OK]");
    }
}
