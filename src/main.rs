#!feature(linked_list_extras)]
extern crate pancurses;

use std::path::{Path,PathBuf};
use std::io::*;
use std::io::prelude::*;
use std::fs::File;
use std::collections::LinkedList;
use pancurses::*;

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
    let window = initscr();
    window.refresh();
    window.keypad(true);
   // window.nodelay(true);
    noecho();

    let mut state = State::init(window);
    state.buffers.push(Buffer::new());
    let mut cur_mode : Box<Mode> = Box::new(mode::NormalMode{});
    while !state.should_quit {
        state.win.clear();
        cur_mode.draw(&state.win);
        state.win.mv(state.win.get_max_y()-2, 0);
        state.win.printw(cur_mode.status_text());
        state.win.addch('|');
        state.win.addch(' ');
        let pth_str = match state.current_buffer().fs_loc {
            Some(ref path) => path.to_string_lossy().into_owned(),
            None => String::from("[New File]")
        };
        state.win.printw(&pth_str);
        state.win.mv(state.win.get_max_y()-2, 0);
        state.win.chgat(-1, A_REVERSE, COLOR_WHITE);
        if let Some(ref e) = state.usr_err {
            state.win.mvprintw(state.win.get_max_y()-1, 0, &format!("{}", e));
            state.win.mv(state.win.get_max_y()-1, 0);
            state.win.chgat(-1, A_COLOR, COLOR_GREEN);
        }
        state.win.mv(0,0);
        state.current_buffer().draw((0,0), &state.win);
        state.win.refresh();

        match state.win.getch() {
            Some(i) => { 
                if state.usr_err.is_some() { state.usr_err = None; }
                let nm = cur_mode.handle_input(i, &mut state); 
                match nm {
                    Some(mode) => { cur_mode = mode },
                    None => ()
                }
            },
            None => ()
        }
    }
    endwin();
}
