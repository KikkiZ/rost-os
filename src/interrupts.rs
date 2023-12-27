use crate::gdt;
use crate::print;
use crate::println;
use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use pc_keyboard::HandleControl;
use pc_keyboard::Keyboard;
use pc_keyboard::ScancodeSet1;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
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

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

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

// 时钟中断
extern "x86-interrupt" fn timer_interrupt_handler(_stack_fram: InterruptStackFrame) {
    print!(".");

    unsafe {
        // 结束中断, 自动返回发送中断信号的源头
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// 键盘中断
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_fram: InterruptStackFrame) {
    // print!("k");

    // unsafe {
    //     PICS.lock()
    //         .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    // }
    use pc_keyboard::layouts;
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key)     => print!("{:?}", key),
            }
        }
    }

    // 读取扫描码, 并输出数据
    // let mut port = Port::new(0x60);
    // let scancode: u8 = unsafe { port.read() };

    // let key = match scancode {
    //     0x02 => Some('1'),
    //     0x03 => Some('2'),
    //     0x04 => Some('3'),
    //     0x05 => Some('4'),
    //     0x06 => Some('5'),
    //     0x07 => Some('6'),
    //     0x08 => Some('7'),
    //     0x09 => Some('8'),
    //     0x0a => Some('9'),
    //     0x0b => Some('0'),
    //     _    => None,
    // };

    // if let Some(key) = key {
    //     print!("{}", key);
    // }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_execption() {
    x86_64::instructions::interrupts::int3();
}

// 处理计时器中断
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
