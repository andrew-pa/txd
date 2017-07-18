extern crate vtctl;
use vtctl::*;

use std::io;
use std::io::Write as IOWrite;
use std::fmt::Write as FmtWrite;

#[test]
fn basic() {
    let mut win = Window::new(); 
    win.clear();
    win.set_cur(0,100);
    win.set_attrib(Color::Red, Color::Green, true, true, false);
    win.write_str("Hello, World!");
    win.clear_attrib();
    'mainloop: loop {
        for i in win.inputs() {
            match i {
                Input::KeyFunc(1) => break 'mainloop,
                Input::KeyRelease => (),
                Input::KeyChar(c) => win.write_char(c).unwrap(),
                _ => {
                    win.set_attrib(Color::Red, Color::Default, true, false, false);
                    write!(win, "{:?}", i).unwrap();
                    win.clear_attrib();
                }
            }
        }
        io::stdout().flush().unwrap();
    }
}
