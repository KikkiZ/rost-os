#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

// 该crate不包含在标准库中, 但它在no_std环境下处于默认禁用状态
extern crate alloc;

use core::panic::PanicInfo;

use allocator::{fixed_size_block::FixedSizeBlockAllocator, Locked};

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

#[global_allocator]
// 使用现有的crate来实现分配器
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
// 使用自定义的Bump分配器
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// 使用自定义的LinkedList分配器
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
// 使用自定义的FixedSizeBlock分配器
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

// 在lib.rs中初始化可以让所有的_start共享初始化逻辑
pub fn init() {
    gdt::init(); // 初始化gdt
    interrupts::init_idt(); // 初始化idt
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // 开启中断
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[OK]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel);

#[cfg(test)]
fn test_kernel(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

// #[cfg(test)]
// #[no_mangle]
// pub extern "C" fn _start() -> ! {
//     init();
//     test_main();
//     hlt_loop();
// }

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
