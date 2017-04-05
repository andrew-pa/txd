extern crate pancurses;

use std::collections::LinkedList;
use pancurses::*;

struct Buffer {
    lines: LinkedList<String>
}

impl Buffer {
    fn new() -> Buffer {
        Buffer { lines: LinkedList::new() }
    }

    fn draw(win: &Window) {
    }
}

fn main() {
  let window = initscr();
  window.refresh();
  window.keypad(true);
  noecho();
  loop {
      match window.getch() {
          Some(Input::Character(c)) => { window.addch(c); },
          Some(Input::KeyDC) => break,
          Some(input) => { window.addstr(&format!("{:?}", input)); },
          None => ()
      }
  }
  endwin();
}
