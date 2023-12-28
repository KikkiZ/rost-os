use crate::gdt;
use crate::hlt_loop;
use crate::print;
use crate::warn;
use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use pc_keyboard::HandleControl;
use pc_keyboard::Keyboard;
use pc_keyboard::ScancodeSet1;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

extern "x86-interrupt" fn breakpoint_handler(stack_fram: InterruptStackFrame) {
    warn!("EXCEPTION: BREAKPOINT\n{:#?}", stack_fram);
}

// 内存越界访问时触发该异常
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    warn!("EXCEPTION: PAGE FAULT");
    warn!("Accessed Address: {:?}", Cr2::read());
    warn!("Error Code: {:?}", error_code);
    warn!("{:#?}", stack_frame);

    hlt_loop();
}

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
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// 键盘中断
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_fram: InterruptStackFrame) {
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
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

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
