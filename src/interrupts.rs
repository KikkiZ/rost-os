use crate::println;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static !{
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

// 使用懒加载, 将idt抽象为一个相对安全的接口
pub fn init_idt() {
    IDT.load();
}

// 初始化函数, 用于创建一个新的中断描述表, 且需要具有整个程序的生命周期
// 对于一个静态类型定义为可变变量是不安全的, 容易形成数据竞争
// static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

// pub fn init_idt() {
//     unsafe {
//         // 加载我们自定义的中断
//         IDT.breakpoint.set_handler_fn(breakpoint_handler);
//         IDT.load();
//     }
// }

// x86-interrupt不是稳定的特性, 需要在lib.rs中手动添加
extern "x86-interrupt" fn breakpoint_handler(stack_fram: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_fram);
}

#[test_case]
fn test_breakpoint_execption() {
    x86_64::instructions::interrupts::int3();
}
