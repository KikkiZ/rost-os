#![no_std] // 告诉编译器不使用标准库, 只使用用核心库
#![no_main] // 删除main函数, 告诉编译器无需初始化

mod vga_buffer;

use crate::vga_buffer::Writer;
use core::panic::PanicInfo;

// 当移除标准库之后, 核心库中不包含panic宏的具体实现
// 因此需要手动实现该模块
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
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

    Writer::print_sth();

    loop {}
}

// fn main() {
// 当我们不适用标准库之后, 输出宏将无法使用
// println!("Hello, world!");
// }
