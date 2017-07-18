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
    let window = Window::new();

    let mut state = State::init(window);
    state.buffers.push(Buffer::new());
    let mut cur_mode : Box<Mode> = Box::new(mode::NormalMode{});
    while !state.should_quit {
        let win_size = state.win.size();
        state.win.clear();
        cur_mode.draw(&state.win);
        state.win.set_cur(0, win_size.1-2);
        state.win.write_str(cur_mode.status_text());
        state.win.write_char('|');
        state.win.write_char(' ');
        let pth_str = match state.current_buffer().fs_loc {
            Some(ref path) => path.to_string_lossy().into_owned(),
            None => String::from("[New File]")
        };
        state.win.write_str(&pth_str);
        state.win.set_cur(0, win_size.1-2);
        //state.win.chgat(-1, A_REVERSE, COLOR_WHITE);
        if let Some(ref e) = state.usr_err {
            state.win.write_at(win_size.1-1, 0, &format!("{}", e));
            state.win.set_cur(0, win_size.1-1);
            //state.win.chgat(-1, A_COLOR, COLOR_GREEN);
        }
        state.win.set_cur(0,0);
        state.current_buffer().draw((0,0), &state.win);
        state.win.refresh();

        for i in state.win.inputs() {
            if state.usr_err.is_some() { state.usr_err = None; }
            let nm = cur_mode.handle_input(i, &mut state); 
            match nm {
                Some(mode) => { cur_mode = mode },
                None => ()
            }
        }
    }
}
