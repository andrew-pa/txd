extern crate pancurses;

use std::collections::LinkedList;
use std::rc::Rc;
use pancurses::*;

/* 
 * Mode
 *  handleInput :: Input, &mut Buffer -> Mode
 *  draw :: Window -> ()
 */

struct State {
    cur_x: i32, cur_y: i32,
    cur_buf: Box<Buffer>, cur_mode: Box<Mode>
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
    fn handle_input(&self, i: Input, s: State) -> State;
    fn draw(&self, win: &Window);
}

struct NormalMode {}
//struct InsertMode {}

impl Mode for NormalMode {
    fn handle_input(&self, i: Input, s: State) -> State {
        match i {
            Input::Character('h') => State { cur_x: s.cur_x - 1, ..s },
            Input::Character('j') => State { cur_x: s.cur_y + 1, ..s },
            Input::Character('k') => State { cur_x: s.cur_y - 1, ..s },
            Input::Character('l') => State { cur_x: s.cur_x + 1, ..s },
            Input::Character('i') => {
     //           s.cur_mode = Rc::new(InsertMode{})
                s
            },
            _ => s
        }
    }
    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y(), 0, "NORMAL");
    }
}
/*
impl Mode for InsertMode {
    fn handle_input(&self, i: Input, s: &mut State) {
        match i {
            Input::Character(c) => {
                if !c.is_control() {
                    s.cur_buf.lines.iter_mut().nth(s.cur_y as usize).unwrap().insert(s.cur_x as usize, c);
                    s.cur_x += 1
                } else if c == '\x08' {
                    s.cur_buf.lines.iter_mut().nth(s.cur_y as usize).unwrap().remove(s.cur_x as usize - 1);
                    s.cur_x -= 1
                }
            },
            _ => ()
        }
    }

    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y(), 0, "INSERT");
    }
}*/

fn main() {
    let window = initscr();
    window.refresh();
    window.keypad(true);
    window.nodelay(true);
    noecho();

    let mut state = State {
        cur_x: window.get_cur_x(), cur_y: window.get_cur_y(),
        cur_buf: Box::new(Buffer::new()), cur_mode: Box::new(NormalMode{})
    };
    state.cur_buf.lines.push_front(String::from("This is the first line in the buffer!"));
    state.cur_buf.lines.push_front(String::from("This is the second line in the buffer!"));
    loop {
        match window.getch() {
            Some(Input::KeyDC) => break,
            Some(i) => {
                let new_state = state.cur_mode.handle_input(i, state);
                state = new_state;
            },
            None => ()
        }
        window.clear();
        state.cur_mode.draw(&window);
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
