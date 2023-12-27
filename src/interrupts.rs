use crate::gdt;
use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // idt.double_fault.set_handler_fn(double_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
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

// double_fault是永远没有返回值的,
// x86_64不允许从该异常中返回任何东西
// double_fault是CPU执行错误处理函数失败时抛出的异常
extern "x86-interrupt" fn double_fault_handler(
    stack_fram: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_fram);
}

#[test_case]
fn test_breakpoint_execption() {
    x86_64::instructions::interrupts::int3();
}
