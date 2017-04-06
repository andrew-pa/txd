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

    fn draw(&self, (x0,y0): (i32,i32), win: &Window) {
        for (y,l) in (y0..).zip(self.lines.iter()) {
            win.mvprintw(y, x0, l);
        }
    }
}

fn main() {
  let window = initscr();
  window.refresh();
  window.keypad(true);
  noecho();

  let mut cur_x = window.get_cur_x();
  let mut cur_y = window.get_cur_y();

 // let subwin = window.subwin(8,40,2,2).unwrap();
  
  //  subwin.border('|','|','-','-','+','+','+','+');
    let mut buf = Buffer::new();
    buf.lines.push_front(String::from("Hello, World! 1"));
    buf.lines.push_front(String::from("Hello, World! 2"));
    buf.lines.push_front(String::from("Hello, World! 3"));
    buf.lines.push_front(String::from("Hello, World! 4"));
  //  subwin.refresh();

  loop {
      match window.getch() {
          Some(Input::KeyUp) => { cur_y -= 1 },
          Some(Input::KeyDown) => { cur_y += 1 },
          Some(Input::KeyLeft) => { cur_x -= 1 },
          Some(Input::KeyRight) => { cur_x += 1 },
          Some(Input::Character(c)) => {
              buf.lines.iter_mut().nth(cur_y as usize).unwrap().insert(cur_x as usize, c);
              cur_x += 1
          },
          Some(Input::KeyDC) => break,
          Some(input) => { window.addstr(&format!("{:?}", input)); },
          None => ()
      }
      buf.draw((0,0), &window);
      window.mv(cur_y, cur_x);
      window.refresh();
  }
  endwin();
}
