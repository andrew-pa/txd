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
    cur_buf: Box<Buffer>, win: Window,
    should_quit: bool
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
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>>;
    fn draw(&self, win: &Window);
}

struct NormalMode {}
struct InsertMode {}
struct CommandMode {
    buf: String, old_cur: (i32,i32)
}

impl Mode for NormalMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
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
            Input::Character(':') => {
                let r : Box<Mode> = Box::new(CommandMode{buf: String::from(""), old_cur:(s.cur_x,s.cur_y)});
                s.cur_x = 0; s.cur_y = s.win.get_max_y()-1;
                Some(r)
            },
            _ => None
        }
    }
    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-1, 0, "NORMAL");
    }
}

impl Mode for InsertMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
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

impl CommandMode {
    fn run_command(&mut self, s: &mut State) -> Option<Box<Mode>> {
        if self.buf.chars().next() == Some('q') { s.should_quit = true; }
        None
    }
}

impl Mode for CommandMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('\x1B') => {
                s.cur_x = self.old_cur.0;
                s.cur_y = self.old_cur.1;
                Some(Box::new(NormalMode{}))
            },
            Input::Character('\n')=> {
                s.cur_x = self.old_cur.0;
                s.cur_y = self.old_cur.1;
                self.run_command(s).or(Some(Box::new(NormalMode{})))
            }
            Input::Character(c) => {
                if !c.is_control() {
                    self.buf.insert(s.cur_x as usize, c);
                    s.cur_x += 1
                } else if c == '\x08' {
                    self.buf.remove(s.cur_x as usize - 1);
                    s.cur_x -= 1
                }
                None
            },
            _ => None
        }
    }

    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-1, 0, &self.buf);
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
        cur_buf: Box::new(Buffer::new()), win: window, should_quit: false
    };
    let mut cur_mode : Box<Mode> = Box::new(NormalMode{});
    state.cur_buf.lines.push_front(String::from("This is the first line in the buffer!"));
    state.cur_buf.lines.push_front(String::from("This is the second line in the buffer!"));
    while !state.should_quit {
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
        state.win.clear();
        cur_mode.draw(&state.win);
        state.cur_buf.draw((0,0), &state.win);
        state.win.mv(state.cur_y, state.cur_x);
        state.win.refresh();
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
