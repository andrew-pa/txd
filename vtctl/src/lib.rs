extern crate winapi;
extern crate kernel32;

use winapi::*;
use kernel32::*;
use std::char::decode_utf16;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::Write as IoWrite;

pub struct Window { 
    out: HANDLE,
    cin: HANDLE,
    cur_x: usize, cur_y: usize
}

pub enum Color {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White, Default
}

impl Color {
    pub fn index(self) -> usize {
        match self {
            Color::Black => 0, 
            Color::Red => 1, 
            Color::Green => 2, 
            Color::Yellow => 3, 
            Color::Blue => 4, 
            Color::Magenta => 5, 
            Color::Cyan => 6, 
            Color::White => 7,
            Color::Default => 9,
        }
    }
}

#[derive(Debug)]
pub enum Input {
    Unknown,
    Character(char),
    KeyEscape,
    KeyFunc(i8),
    KeyRelease,
    KeyUp, KeyDown, KeyLeft, KeyRight, KeyBackspace, KeyEnter
}

impl Input {
    pub fn from_win32(inp: INPUT_RECORD) -> Input {
        match inp.EventType {
            KEY_EVENT => {
                let ker = unsafe { inp.KeyEvent() };
                if ker.bKeyDown != 1 { Input::KeyRelease } else {
                match ker.wVirtualKeyCode as i32 {
                    VK_ESCAPE => Input::KeyEscape,
                    VK_UP => Input::KeyUp,
                    VK_DOWN => Input::KeyDown,
                    VK_LEFT => Input::KeyLeft,
                    VK_RIGHT => Input::KeyRight,
                    VK_BACK => Input::KeyBackspace,
                    VK_RETURN => Input::KeyEnter,
                    VK_F1 => Input::KeyFunc(1),
                    _=> { 
                        let v = &[ker.UnicodeChar];
                        let mut dv = decode_utf16(v.iter().cloned());
                        Input::Character(dv.next().unwrap().unwrap())
                    }
                } }
            },
            _ => Input::Unknown
        }
    }
}

pub struct InputQueue {
    cin: HANDLE,
    queue: Vec<Input>
}

impl Window {
    pub fn new() -> Window {
        unsafe {
            let mut out = GetStdHandle(STD_OUTPUT_HANDLE); 
            let mut cin = GetStdHandle(STD_INPUT_HANDLE); 
            let mut mode: DWORD = 0;
            GetConsoleMode(out, &mut mode);
            SetConsoleMode(out, mode | 0x0004);
            print!("\x1b[0G\x1b[0d");
            Window { out: out, cin: cin, cur_x: 0, cur_y: 0 }
        }
    }

    pub fn clear_attrib(&self) {
        print!("\x1b[0m");
    }
    pub fn set_attrib(&self, fg: Color, bg: Color, bright_fg: bool, bright_bg: bool, underline: bool) {
        let vfg = if bright_fg { 90 } else { 30 } + fg.index(); 
        let vbg = if bright_bg { 100 } else { 40 } + bg.index(); 
        print!("\x1b[{};{}m", vfg,vbg);//, vbg, if underline { 4 } else { 24 });
    }
    
    pub fn clear(&mut self) {
        print!("\x1b[2J");
        self.set_cur(0,0);
    }

    pub fn set_cur(&mut self, x: usize, y: usize) {
        self.cur_x = x;
        self.cur_y = y;
        unsafe {
            SetConsoleCursorPosition(self.out, COORD {X: x as i16, Y: y as i16});
        }
        //print!("\x1b[{},{}H", y, x);
    }

    pub fn write_at(&mut self, y: usize, x: usize, s: &str) {
        self.set_cur(x, y);
        self.write_str(s);
    }

    pub fn size(&self) -> (usize,usize) {
        unsafe {
            use std::mem::uninitialized;
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = uninitialized();
            GetConsoleScreenBufferInfo(self.out, &mut info);
            (info.dwSize.X as usize, info.dwSize.Y as usize)
        }
    }

    pub fn inputs(&self) -> InputQueue {
        InputQueue {cin: self.cin, queue: Vec::new()}
    }

    pub fn refresh(&self) {
        io::stdout().flush().expect("flushing stdout");
    }
}

use std::ptr::{null_mut, null};
impl FmtWrite for Window {
    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        self.cur_x += 1;
        let mut buf = [0u16, 3];
        c.encode_utf16(&mut buf);
        unsafe {
            WriteConsoleW(self.out, buf.as_ptr() as *const std::os::raw::c_void, 3u32, null_mut(), null_mut());
        }
        Ok(())
    }
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.cur_x += s.len();
        unsafe {
            WriteConsoleW(self.out, s.encode_utf16().collect::<Vec<_>>().as_ptr() as *const std::os::raw::c_void, s.len() as u32, null_mut(), null_mut());
        }
        Ok(())
    }
    fn write_fmt(&mut self, args: fmt::Arguments) -> Result<(), fmt::Error> {
        self.write_str(&fmt::format(args))
    }
}

impl Iterator for InputQueue {
    type Item = Input;

    fn next(&mut self) -> Option<Input> {
        if self.queue.is_empty() {
            let new_records = &mut [INPUT_RECORD{EventType: 0, Event:[0,0,0,0]}; 128];
            let mut num_rec: DWORD = 0;
            unsafe {
                PeekConsoleInputW(self.cin, new_records.as_mut_ptr(), new_records.len() as u32, &mut num_rec);
            }
            if num_rec > 0 {
                unsafe { FlushConsoleInputBuffer(self.cin); }
                for i in 0..num_rec {
                    self.queue.push(Input::from_win32(new_records[i as usize]));
                }
                self.queue.pop()
            } else {
                None
            }
        } else {
            self.queue.pop()
        }
    }
}


