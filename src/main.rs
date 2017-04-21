#![feature(linked_list_extras)]
extern crate pancurses;

use std::path::{Path,PathBuf};
use std::io::*;
use std::io::prelude::*;
use std::fs::File;
use std::collections::LinkedList;
use pancurses::*;

/* 
 * Mode
 *  handleInput :: Input, &mut Buffer -> Mode
 *  draw :: Window -> ()
 */

struct State {
    cur_x: usize, cur_y: usize,
    cur_buf: Box<Buffer>, win: Window,
    should_quit: bool
}

struct Buffer {
    fs_loc: PathBuf,
    file: Option<File>,
    lines: LinkedList<String>
}

impl Buffer {
    fn new() -> Buffer {
        Buffer { fs_loc: PathBuf::new(), file: None, lines: LinkedList::new() }
    }

    fn load(fp: &Path) -> Buffer {
        let mut f = File::open(fp).unwrap();
        let mut s : String = String::new();
        f.read_to_string(&mut s);
        let mut buf = Buffer {
            fs_loc: PathBuf::from(fp),
            file: Some(f),
            lines: LinkedList::new()
        };
        for ln in s.lines() {
            buf.lines.push_back(String::from(ln));
        }
        buf
    }

    fn insert_line(&mut self, y: usize, s: Option<String>) {
        let mut back = self.lines.split_off(y);
        self.lines.push_back(s.unwrap_or(String::from("")));
        self.lines.append(&mut back);
    }

    fn insert_char(&mut self, c: char, (x,y): (usize,usize)) {
        self.lines.iter_mut().nth(y).unwrap().insert(x, c);
    }

    fn remove_char(&mut self, (x,y): (usize,usize)) {
        self.lines.iter_mut().nth(y).unwrap().remove(x);
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
    buf: String, old_cur: (usize,usize)
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
                s.cur_buf.insert_line(s.cur_y, None);
                Some(Box::new(InsertMode{}))
            },
            Input::Character(':') => {
                let r : Box<Mode> = Box::new(CommandMode{buf: String::from(""), old_cur:(s.cur_x,s.cur_y)});
                s.cur_x = 0; s.cur_y = s.win.get_max_y()as usize-1;
                Some(r)
            },
            _ => None
        }
    }
    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-2, 0, "NORMAL");
    }
}

impl Mode for InsertMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('\x1B') => Some(Box::new(NormalMode{})),
            Input::Character(c) => {
                if !c.is_control() {
                    s.cur_buf.insert_char(c,(s.cur_x, s.cur_y));
                    s.cur_x += 1
                } else if c == '\x08' {
                    s.cur_buf.remove_char((s.cur_x - 1, s.cur_y));
                    s.cur_x -= 1
                }
                None
            },
            _ => None
        }
    }

    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-2, 0, "INSERT");
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
                    self.buf.insert(s.cur_x, c);
                    s.cur_x += 1
                } else if c == '\x08' {
                    self.buf.remove(s.cur_x - 1);
                    s.cur_x -= 1
                }
                None
            },
            _ => None
        }
    }

    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-2, 0, "COMMAND");
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
        cur_x: window.get_cur_x() as usize, cur_y: window.get_cur_y() as usize,
        cur_buf: Box::new(Buffer::load(Path::new("C:\\Users\\andre\\Source\\txd\\src\\main.rs"))), win: window, should_quit: false
    };
    let mut cur_mode : Box<Mode> = Box::new(NormalMode{});
    /*state.cur_buf.lines.push_front(String::from("This is the first line in the buffer!"));
    state.cur_buf.lines.push_front(String::from("This is the second line in the buffer!"));*/
    while !state.should_quit {
        state.win.clear();
        cur_mode.draw(&state.win);
        state.win.mv(state.win.get_max_y()-2, 0);
        state.win.chgat(-1, A_REVERSE, COLOR_WHITE);
        //state.win.hline('_', 10000);
        state.win.mv(0,0);
        state.cur_buf.draw((0,0), &state.win);
        state.win.mv(state.cur_y as i32, state.cur_x as i32);
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
