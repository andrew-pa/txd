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
    lines: Vec<String>,
    cur_line: usize,
    cur_col: usize,
    viewport_line: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer { fs_loc: PathBuf::new(), file: None, lines: Vec::new(), cur_line:0, cur_col:0, viewport_line:0 }
    }

    fn load(fp: &Path) -> Buffer {
        let mut f = File::open(fp).unwrap();
        let mut s : String = String::new();
        f.read_to_string(&mut s);
        let mut buf = Buffer {
            fs_loc: PathBuf::from(fp),
            file: Some(f),
            lines: s.lines().map(String::from).collect(),
            cur_line:0,cur_col:0,viewport_line:0
        };
        /*for ln in s.lines() {
            buf.lines.push_back(String::from(ln));
        }*/
        buf
    }

    fn set_loca(&mut self, col: usize, line: usize) {
        self.cur_line = if line >= self.lines.len() { self.lines.len().saturating_sub(1) } else { line };
        assert!(self.cur_line < self.lines.len());
        self.cur_col = if col > self.lines[self.cur_line].len() { self.lines[self.cur_line].len().saturating_sub(1) } else { col };
        if self.cur_line < self.viewport_line {
            self.viewport_line = self.cur_line;
        } else if self.cur_line > self.viewport_line+22 {
            self.viewport_line = self.cur_line;
        }
    }

    fn move_loca(&mut self, dx: isize, dy: isize) {
        let x = self.cur_col as isize + dx; let y = self.cur_line as isize + dy;
        self.set_loca(if x < 0 { 0 } else { x as usize }, if y < 0 { 0 } else { y as usize });
    }

    fn insert_lined(&mut self, y: usize, s: Option<String>) {
        self.lines.insert(y,s.unwrap_or(String::from("")));
    }

    fn insert_chard(&mut self, c: char, (x,y): (usize,usize)) {
        self.lines[y].insert(x,c);
    }

    fn remove_chard(&mut self, (x,y): (usize,usize)) {
        self.lines[y].remove(x);
    }


    fn insert_char(&mut self, c: char) {
        let p =(self.cur_col, self.cur_line);
        self.insert_chard(c, p);
        self.move_loca(1,0);
    }

    fn backspace(&mut self) {
        let p =(self.cur_col-1, self.cur_line); 
        self.remove_chard(p);
        self.move_loca(-1,0);
    }

    fn insert_line(&mut self, s: Option<String>) {
        let y = self.cur_line;
        self.insert_lined(y+1, s);
        self.set_loca(0,y+1);
    }

    fn draw(&self, (x0,y0): (i32,i32), win: &Window) {
        for (y,l) in (y0..).zip(self.lines.iter().skip(self.viewport_line).take(win.get_max_y()as usize-2)) {
            win.mvprintw(y, x0, l);
        }
        win.mv((self.cur_line - self.viewport_line) as i32, self.cur_col as i32);
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
            Input::Character('h') => { s.cur_buf.move_loca(-1,  0); None },
            Input::Character('j') => { s.cur_buf.move_loca( 0,  1); None },
            Input::Character('k') => { s.cur_buf.move_loca( 0, -1); None },
            Input::Character('l') => { s.cur_buf.move_loca( 1,  0); None },
            Input::Character('i') => Some(Box::new(InsertMode{})),
            Input::Character('o') => { 
                s.cur_buf.insert_line(None);
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
                    s.cur_buf.insert_char(c);
                } else if c == '\x08' {
                    s.cur_buf.backspace();
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
        //state.win.mv(state.cur_y as i32, state.cur_x as i32);
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
