#![feature(linked_list_extras)]
extern crate pancurses;

use std::collections::LinkedList;
use pancurses::*;

/* 
 * Mode
 *  handleInput :: Input, &mut Buffer -> Mode
 *  draw :: Window -> ()
 */

struct State {
    cur_x: i32, cur_y: i32,
    cur_buf: Box<Buffer>
}

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

trait Mode {
    fn handle_input(&self, i: Input, s: &mut State) -> Option<Box<Mode>>;
    fn draw(&self, win: &Window);
}

struct NormalMode {}
struct InsertMode {}

impl Mode for NormalMode {
    fn handle_input(&self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('h') => { s.cur_x -= 1; None },
            Input::Character('j') => { s.cur_y += 1; None },
            Input::Character('k') => { s.cur_y -= 1; None },
            Input::Character('l') => { s.cur_x += 1; None },
            Input::Character('i') => Some(Box::new(InsertMode{})),
            Input::Character('o') => { 
                s.cur_x = 0; s.cur_y += 1;
                let mut back = s.cur_buf.lines.split_off(s.cur_y as usize);
                s.cur_buf.lines.push_back(String::from(""));
                s.cur_buf.lines.append(&mut back);
                Some(Box::new(InsertMode{}))
            },
            _ => None
        }
    }
    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-1, 0, "NORMAL");
    }
}

impl Mode for InsertMode {
    fn handle_input(&self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('\x1B') => Some(Box::new(NormalMode{})),
            Input::Character(c) => {
                if !c.is_control() {
                    s.cur_buf.lines.iter_mut().nth(s.cur_y as usize).unwrap().insert(s.cur_x as usize, c);
                    s.cur_x += 1
                } else if c == '\x08' {
                    s.cur_buf.lines.iter_mut().nth(s.cur_y as usize).unwrap().remove(s.cur_x as usize - 1);
                    s.cur_x -= 1
                }
                None
            },
            _ => None
        }
    }

    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-1, 0, "INSERT");
    }
}

fn main() {
    let window = initscr();
    window.refresh();
    window.keypad(true);
   // window.nodelay(true);
    noecho();

    let mut state = State {
        cur_x: window.get_cur_x(), cur_y: window.get_cur_y(),
        cur_buf: Box::new(Buffer::new())
    };
    let mut cur_mode : Box<Mode> = Box::new(NormalMode{});
    state.cur_buf.lines.push_front(String::from("This is the first line in the buffer!"));
    state.cur_buf.lines.push_front(String::from("This is the second line in the buffer!"));
    loop {
        match window.getch() {
            Some(Input::KeyDC) => break,
            Some(i) => { 
                let nm = cur_mode.handle_input(i, &mut state); 
                match nm {
                    Some(mode) => { cur_mode = mode },
                    None => ()
                }
            },
            None => ()
        }
        window.clear();
        cur_mode.draw(&window);
        state.cur_buf.draw((0,0), &window);
        window.mv(state.cur_y, state.cur_x);
        window.refresh();
    }
    endwin();
}
/*
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
          Some(Input::KeyBackspace) => {
              buf.lines.iter_mut().nth(cur_y as usize).unwrap().remove(cur_x as usize);
              cur_x -= 1
          },
          Some(Input::Character(c)) => {
              if !c.is_control() {
                buf.lines.iter_mut().nth(cur_y as usize).unwrap().insert(cur_x as usize, c);
                cur_x += 1
              } else if c == '\x08' {
                buf.lines.iter_mut().nth(cur_y as usize).unwrap().remove(cur_x as usize - 1);
                cur_x -= 1
              }
          },
          Some(Input::KeyDC) => break,
          Some(input) => { window.addstr(&format!("{:?}", input)); },
          None => ()
      }
      window.clear();
      buf.draw((0,0), &window);
      window.mv(cur_y, cur_x);
      window.refresh();
  }
  endwin();
}*/
