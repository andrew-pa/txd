
extern crate rustbox;

use std::error::Error;
use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;

struct Buffer {
    lines : Vec<String>,
    cur_idx: usize,
    cur_col: usize,
    cur_row: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer { 
            lines: vec![String::new()], 
            cur_idx:0, 
            cur_col:0,
            cur_row:0,
        }
    }

    fn insert(&mut self, c : char) {
        let cr = self.cur_row;
        let ci = self.cur_idx;
        if ci == self.lines[cr].len() {
            self.lines[cr].push(c);
        } else {
            self.lines[cr].insert(ci, c);
        }
        self.cur_col += 1;
        self.cur_idx += c.len_utf8(); //Strings are UTF8
    }

    fn new_line(&mut self) {
        self.lines.push(String::new());
        self.cur_row += 1;
        self.cur_idx = 0;
        self.cur_col = 0;
    }

    fn draw(&self, x: usize, y: usize, rustbox: &RustBox) {
        let mut cy = y;
        for ln in &self.lines {
            rustbox.print(x, cy, rustbox::RB_NORMAL, Color::White, Color::Black, &ln);
            cy += 1;
        }
        rustbox.set_cursor(x+self.cur_col as isize, y+self.cur_row as isize);
    }
}

fn main() {
    let rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    let mut buf = Buffer::new();
    buf.new_line();
    let mut buf_changed = false;
    //rustbox.print(1, 3, rustbox::RB_BOLD, Color::White, Color::Black,
    //             "Press 'q' to quit.");
    rustbox.present();
    loop {
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char(c) => { 
                        buf.insert(c);
                        buf_changed = true;
                    }
                    Key::Enter => {
                        buf.new_line();
                        buf_changed = true;
                    }
                    Key::Left => {
                    }
                    Key::Esc => { break; }
                    _ => { }
                }
            },
            Err(e) => panic!("{}", e.description()),
            _ => { }
        }
        if buf_changed {
            buf_changed = false;
            rustbox.clear();
            buf.draw(1,1,&rustbox);
//rustbox.print(1, 1, rustbox::RB_NORMAL, Color::Green, Color::Black, &buf);
            rustbox.present();
        }
    }
}
