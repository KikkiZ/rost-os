use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// 实现了一个串口模块, 用于将测试信息返回到控制台
lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// 用于串口输出信息
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
