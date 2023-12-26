// dead_code 禁止编译器对未使用的函数产生警告
#[allow(dead_code)]
// 编译器自动实现某些trait
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 定义内存的布局方式, 表明每个枚举变量占一字节
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // 确保类型和它的单个成员有相同的内存布局
struct ColorCode(u8);

// 包装了一个完整的颜色代码字节, 包含前景色和背景色
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // 左移四位, 然后进行按位或运算
        ColorCode((background as u8) << 4 | foreground as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 使用C约定的顺序布局成员变量, 确保正确的映射到内存片段
           // 描述字符
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// 引入Volatile包, 它提供了read和write方法,
// 该方法保证读写操作不会被编译器优化
use volatile::Volatile;

#[repr(transparent)]
// 描述了整个字符缓冲区
struct Buffer {
    // chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    // 使用volatile提供的方法重写VGA的写入
    // 同时使用泛型, 防止意外的写入数据
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// 将字符写入屏幕的最后一行,
// 并在一行写满或收到换行符时将所有字符上移一行
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    // 声明一个全局且整个运行期间有效的缓冲区
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // 判断是否超出最大宽度, 是则换行
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                // 确定当前字符的位置
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // self.buffer.chars[row][col] = ScreenChar {
                //     ascii_character: byte,
                //     color_code,
                // };
                // 使用volatile提供的write方法
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 匹配能够输出的字符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 其他字符则替换
                _ => self.write_byte(0xfe),
            }
        }
    }

    // 实现换行方法
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // 读取从第一行开始的数据
                let character = self.buffer.chars[row][col].read();
                // 将该数据替换到上一行
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        // 清空最后一行的数据
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        // 定义一个空字符
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        // 将指定行替换为空字符
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    // 这是一个简单的测试实例, 大概现在没有用了
    // pub fn print_sth() {
    //     use core::fmt::Write;
    //     let mut writer = Writer {
    //         column_position: 0,
    //         color_code: ColorCode::new(Color::LightGray, Color::Black),
    //         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    //     };

    //     writer.write_byte(b'H');
    //     writer.write_string("ello ");
    //     writer.write_string("World!");

    //     write!(writer, "Here are some numbers: {} and {}", 32, 1.0 / 3.0).unwrap();
    // }
}

use core::fmt;

// 为Writer实现core::fmt::Write接口
// 实现了Rust提供的格式化宏, 便于打印变量
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(()) // 该值属于Result枚举类
    }
}

// 创造一个静态的全局变量
// 静态变量在编译时初始化, ColorCode::new应该转换成const函数, 否则将会报错
// 我们使用延迟初始化(lazy_static包), 它提供了lazy_static宏,
// 它使变量不必再编译时初始化, 而在程序运行时执行初始化代码
use lazy_static::lazy_static;
// 标准库提供了互斥锁Mutex, 这使我们能便捷的实现同步, 但我们无法使用
// 我们只能使用简单的自旋锁spinlock, 在一个无限循环中尝试获得这个锁
use spin::Mutex;

lazy_static! {
    // 当前变量还是不可变变量, 实际的用处并不大
    // 静态可变变量被认为是极不安全的, 一些人认为应该删除它们
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// 此处使用了标准库中类似的代码
// print宏调用了该模块中实现的方法
#[macro_export]
macro_rules! print {
    // 使用美元符号在宏系统中声明一个变量来匹配该模式的rust代码
    // 该符号表明这是一个宏变量而不是一个普通变量, 之后的($标识符:捕获方式)将
    // 输入的东西识别, 其中$crate是个特殊的符号, 表明当前目录
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

// println宏调用了print宏
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// 上面的宏能够在模块外访问, 因此它们也应该能够访问_print函数,
// 所以这个函数必须是公有的, 但事实上这是一个私有的实现细节, 通过
// 添加doc(hidden)属性防止在文档中生成该函数
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println() {
    println!("test_println output");
}

#[test_case]
fn test_printlns() {
    for line in 0..200 {
        println!("[line {}]test_println output", line);
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
