use std::path::{Path,PathBuf};
use std::io::*;
use std::io::prelude::*;
use std::fs::*;
use std::collections::LinkedList;
use pancurses::Window;

pub struct Buffer {
    pub fs_loc: PathBuf,
    file: Option<File>,
    lines: Vec<String>,
    cur_line: usize,
    cur_col: usize,
    viewport_line: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer { fs_loc: PathBuf::new(), file: None, lines: Vec::new(), cur_line:0, cur_col:0, viewport_line:0 }
    }

    pub fn load(fp: &Path) -> Buffer {
        let fp_exists = fp.exists();
        let mut f = OpenOptions::new().read(true).write(true).create(true).open(fp).unwrap();
        let lns = if fp_exists { 
            let mut s : String = String::new();
            f.read_to_string(&mut s);
            s.lines().map(String::from).collect()
        } else {
            vec![String::from("~ new file ~")]
        };
        let mut buf = Buffer {
            fs_loc: PathBuf::from(fp),
            file: Some(f),
            lines: lns,
            cur_line:0,cur_col:0,viewport_line:0
        };
        buf
    }

    pub fn sync_disk(&mut self) {
        let lines = self.lines.iter();
        match self.file {
            Some(ref mut f) => {
                f.set_len(0); //truncate the file
                for ln in lines {
                    write!(f, "{}\n", ln);
                }
                f.sync_all();
            },
            None => { panic!("sync_disk with no file backing"); }
        }
    }

    pub fn set_loca(&mut self, col: usize, line: usize) {
        self.cur_line = if line >= self.lines.len() { self.lines.len().saturating_sub(1) } else { line };
        assert!(self.cur_line < self.lines.len());
        self.cur_col = if col > self.lines[self.cur_line].len() { self.lines[self.cur_line].len().saturating_sub(1) } else { col };
        if self.cur_line < self.viewport_line {
            self.viewport_line = self.cur_line;
        } else if self.cur_line > self.viewport_line+22 {
            self.viewport_line = self.cur_line-22;
        }
    }

    pub fn move_loca(&mut self, dx: isize, dy: isize) {
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


    pub fn insert_char(&mut self, c: char) {
        let p =(self.cur_col, self.cur_line);
        self.insert_chard(c, p);
        self.move_loca(1,0);
    }

    pub fn backspace(&mut self) {
        let p =(self.cur_col-1, self.cur_line); 
        self.remove_chard(p);
        self.move_loca(-1,0);
    }
    pub fn break_line(&mut self) {
        if self.cur_col < self.lines[self.cur_line].len() {
            self.insert_line(Some(self.lines[self.cur_line].split_off(self.cur_col)));
        } else {
            self.insert_line(None);
        }
    }

    pub fn insert_line(&mut self, s: Option<String>) {
        let y = self.cur_line;
        self.insert_lined(y+1, s);
        self.set_loca(0,y+1);
    }

    pub fn draw(&self, (x0,y0): (i32,i32), win: &Window) {
        for (y,l) in (y0..).zip(self.lines.iter().skip(self.viewport_line).take(win.get_max_y()as usize-2)) {
            win.mvprintw(y, x0, l);
        }
        win.mv((self.cur_line - self.viewport_line) as i32, self.cur_col as i32);
    }
}
