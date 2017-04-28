#![feature(linked_list_extras)]
extern crate pancurses;

use std::path::{Path,PathBuf};
use std::io::*;
use std::io::prelude::*;
use std::fs::File;
use std::collections::LinkedList;
use pancurses::*;


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
    state.buffers.push(Buffer::new());//Buffer::load(Path::new("C:\\Users\\andre\\Source\\txd\\src\\main.rs")));
    let mut cur_mode : Box<Mode> = Box::new(mode::NormalMode{});
    while !state.should_quit {
        state.win.clear();
        cur_mode.draw(&state.win);
        state.win.mv(state.win.get_max_y()-2, 0);
        state.win.printw(cur_mode.status_text());
        state.win.addch('|');
        state.win.addch(' ');
        state.win.printw(&state.current_buffer().fs_loc.to_string_lossy().into_owned());
        state.win.mv(state.win.get_max_y()-2, 0);
        state.win.chgat(-1, A_REVERSE, COLOR_WHITE);
        state.win.mv(0,0);
        state.current_buffer().draw((0,0), &state.win);
        state.win.refresh();

        match state.win.getch() {
            Some(i) => { 
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
