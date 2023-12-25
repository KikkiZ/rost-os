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

#[repr(transparent)]
// 描述了整个字符缓冲区
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
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

    fn new_line(&mut self) {
        // TODO
    }

    pub fn print_sth() {
        let mut writer = Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::LightGray, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };

        writer.write_byte(b'H');
        writer.write_string("ello ");
        writer.write_string("World!");
    }
}
