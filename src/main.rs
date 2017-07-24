extern crate vtctl;

use std::path::{Path,PathBuf};
use std::io::*;
use std::io::prelude::*;
use std::fs::File;
use std::collections::LinkedList;
use vtctl::*;
use std::fmt::Write as FmtWrite;

/*
 * Need:
 *  + Error handling!!
 *  + Buffer indicator/listing
 *  + text objects + better parsers for both normal&command mode commands
 *  + deletion/change/replace
 *  + custom virtual terminal control because pancurses incredibly questionable (Windows!)
 *  + better handling for line wrapping
 *  + find/replace/switch/regex
 *  + syntax highlighting
 */

mod buffer;
use buffer::*;

mod mode;
use mode::*;

fn main() {
    let mut window = Window::new();
    let mut state = State::init();
    state.buffers.push(Buffer::new());
    let mut cur_mode : Box<Mode> = Box::new(mode::NormalMode{});
       // window.write_at(3,6,"Hello, World!");
    while !state.should_quit {
        for i in window.inputs() {
            let win_size = window.size();
            window.clear();
            cur_mode.draw(&mut window);
            window.set_cur(0, win_size.1-2);
            window.write_str(cur_mode.status_text());
            window.write_char('|');
            window.write_char(' ');
            let pth_str = match state.current_buffer().fs_loc {
                Some(ref path) => path.to_string_lossy().into_owned(),
                None => String::from("[New File]")
            };
            window.write_str(&pth_str);
            window.set_cur(0, win_size.1-2);
            //window.chgat(-1, A_REVERSE, COLOR_WHITE);
            if let Some(ref e) = state.usr_err {
                window.write_at(win_size.1-1, 0, &format!("{}", e));
                window.set_cur(0, win_size.1-1);
                //window.chgat(-1, A_COLOR, COLOR_GREEN);
            }
            window.set_cur(0,0);
            state.current_buffer().draw((0,0), &mut window);
            window.refresh();


            if state.usr_err.is_some() { state.usr_err = None; }
            let nm = cur_mode.handle_input(i, &mut state); 
            match nm {
                Some(mode) => { cur_mode = mode },
                None => ()
            }
        }
    }
}
