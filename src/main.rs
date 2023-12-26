#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;
use rust_os::println;

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {

    println!("Hello World{}", "!");
    println!("Hello RUST!");

    #[cfg(test)]
    test_main();

    loop {}
}
