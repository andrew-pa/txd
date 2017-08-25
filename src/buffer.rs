use std::rc::Rc;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::fs::*;
use std::io::{Read, Write, Error as IoError, ErrorKind};

use runic::{App, Window as SystemWindow, Event, RenderContext, Color, Point, Rect, Font, TextLayout, KeyCode};
use res::Resources;
use movement::Movement;

pub struct Buffer {
    pub fs_loc: Option<PathBuf>,
    pub lines: Vec<String>,

    // buffer view
    pub res: Rc<RefCell<Resources>>,
    line_layouts: Vec<Option<TextLayout>>,
    viewport_start: usize,
    viewport_end: usize,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub show_cursor: bool
}

impl Buffer {
    pub fn new(res: Rc<RefCell<Resources>>) -> Buffer {
        Buffer {
            fs_loc: None, lines: vec![String::from("")],
            res, cursor_line: 0, cursor_col: 0, viewport_start: 0, viewport_end: 0,
            line_layouts: vec![None], show_cursor: true
        }
    }

    pub fn load(fp: &Path, res: Rc<RefCell<Resources>>) -> Result<Buffer, IoError> {
        let fp_exists = fp.exists();
        let (lns, lay) = if fp_exists { 
            let mut f = OpenOptions::new().read(true).write(true).open(fp)?;
            let mut s : String = String::new();
            f.read_to_string(&mut s)?;
            let lns: Vec<String> = s.lines().map(String::from).collect();
            let mut layouts = Vec::new();
            for _ in 0..lns.len() { //replace with Vec::resize_default?
                layouts.push(None);
            }
            (lns, layouts)
        } else {
            (vec![String::from("")], vec![None])
        };
        let mut buf = Buffer {
            fs_loc: Some(PathBuf::from(fp)),
            lines: lns, line_layouts: lay, viewport_start: 0, viewport_end: 0, cursor_line: 0, cursor_col: 0, show_cursor: true, res
        };
        Ok(buf)
    }

    pub fn move_cursor(&mut self, (dx, dy): (isize,isize)) {
        let mut cursor_col = self.cursor_col as isize + dx;
        let mut cursor_line = self.cursor_line as isize + dy;
        if cursor_col < 0 { cursor_col = 0; }
        if cursor_line < 0 { cursor_line = 0; }

        let bl = &self.lines;
        if cursor_line >= bl.len() as isize { cursor_line = (bl.len()-1) as isize; }

        let cln = &bl[cursor_line as usize];
        if cursor_col as usize > cln.len() { cursor_col = cln.len() as isize; }
        while !cln.is_char_boundary(cursor_col as usize) { println!("{}", cursor_col); cursor_col += dx.signum(); }


        if cursor_line < self.viewport_start as isize {
            self.viewport_start = self.viewport_start.saturating_sub(3);
        }
        if cursor_line >= self.viewport_end as isize {
            self.viewport_start = self.viewport_start.saturating_add(3);
        }

        self.cursor_col = cursor_col as usize;
        self.cursor_line = cursor_line as usize;
    }

    pub fn curr_loc(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_line)
    }

    // scan from cursor looking for character. possibly absurdly made and could be done better with
    // a better buffer representation
    pub fn scan_line<P: Fn(char)->bool>(&self, pred: P, forwards: bool) -> Option<isize> {
        let (left, right) = self.lines[self.cursor_line].split_at(self.cursor_col + if forwards {1} else {0});
        //println!("({}, {})", left, right);
        (if forwards { right.find(pred).map(|v| v as isize + 1) } else { left.rfind(pred).map(|v| -(left.len() as isize - v as isize)) })
    }

    pub fn movement_cursor_offset(&mut self, mv: Movement) -> (isize, isize) {
        //println!("movement = {:?}", mv);
        match mv {
            Movement::Char(right) => (if right {1} else {-1}, 0),
            Movement::Line(up) => (0, if up {-1} else {1}),
            Movement::WholeLine => (0, 1),
            Movement::CharScan { query, forwards, place_besides } => {
                match self.scan_line(|q| q==query, forwards) {
                    Some(col) => (col + if place_besides { if forwards {-1} else {1} } else {0}, 0),
                    None => (0,0)
                }
            }
            Movement::Word(forwards, place_at_end) => {
                // this is preliminary. code reuse is sad (copy-pasta from scan_line); additionally
                // the definition of a word may change. Also new, more effecient buffer
                // representations may make this operation much simpler/different
                let pred = |q| !(char::is_alphanumeric(q) || q == '_');
                let (left, right) = self.lines[self.cursor_line].split_at((self.cursor_col as isize + if forwards {1} else {-1}) as usize);
                //println!("({}, {})", left, right);
                match if forwards { right.find(pred).map(|v| v as isize + 1) } else { left.rfind(pred).map(|v| -(left.len() as isize - v as isize + 1)) } {
                    Some(col) => (col + if place_at_end { if forwards {1} else {-1} } else {0}, 0),
                    None => (0,0)
                }
            },
            Movement::EndOfLine => (self.lines[self.cursor_line].len() as isize-self.cursor_col as isize, 0),
            Movement::StartOfLine => (0,0),
            Movement::Rep(count, movement) => {
                let mut offset = (0,0);
                for _ in 0..count {
                    let (dx, dy) = self.movement_cursor_offset(*movement.clone());
                    offset.0 += dx; offset.1 += dy;
                }
                offset
            }
        }
    }

    pub fn make_movement(&mut self, mv: Movement) {
        let offset = self.movement_cursor_offset(mv);
        //println!("offset = {:?}", offset);
        self.move_cursor(offset)
    }

    pub fn delete_movement(&mut self, mv: Movement) {
        match mv {
            Movement::WholeLine => {
                self.lines.remove(self.cursor_line);
                self.line_layouts.remove(self.cursor_line);
            },
            Movement::Rep(count, box Movement::WholeLine) => {
                self.lines.drain(self.cursor_line..(self.cursor_line+count));
                self.line_layouts.drain(self.cursor_line..(self.cursor_line+count));
            },
            _ => {
                let offset = self.movement_cursor_offset(mv);
                if offset.1 == 0 { //deleting within the current line
                    let last = (offset.0 + self.cursor_col as isize) as usize;
                    println!("deleting: {}, {}", self.cursor_col, last);
                    self.lines[self.cursor_line].drain(if self.cursor_col > last { last..self.cursor_col } else { self.cursor_col..last });
                    self.line_layouts[self.cursor_line] = None;
                } else {
                    panic!("r i p");
                }
            }
        }
        self.move_cursor((0,0)); //ensure that the cursor is in a valid position
    }

    pub fn clear(&mut self) {
        self.cursor_col = 0; self.cursor_line = 0;
        self.lines.clear(); self.line_layouts.clear();
        self.lines = vec![String::from("")];
        self.line_layouts = vec![None];
    }

    pub fn invalidate_line(&mut self, line: usize) {
        self.line_layouts[line] = None;
    }

    pub fn insert_char(&mut self, loc: (usize,usize), c: char) {
        self.lines[loc.1].insert(loc.0, c);
        self.invalidate_line(loc.1);
    }
    pub fn delete_char(&mut self, loc: (usize, usize)) {
        if loc.0 >= self.lines[loc.1].len() {
            self.lines[loc.1].pop();
        }
        else {
            self.lines[loc.1].remove(loc.0);
        }
        self.invalidate_line(loc.1);
    }
    pub fn break_line(&mut self, loc: (usize, usize)) {
        let new_line = if loc.0 >= self.lines[loc.1].len() {
            String::from("")
        } else {
            self.lines[loc.1].split_off(loc.0)
        };
        self.lines.insert(loc.1+1, new_line);
        self.invalidate_line(loc.1);
        self.line_layouts.insert(loc.1, None);
        self.cursor_col = 0; self.move_cursor((0,1));
    }
    pub fn insert_line(&mut self, loc: usize) {
        self.lines.insert(loc+1, String::from(""));
        self.line_layouts.insert(loc+1, None);
        self.cursor_col = 0; self.move_cursor((0,1));
    }

    pub fn sync_disk(&mut self) -> Result<(), IoError> {
        let lines = self.lines.iter();
        match self.fs_loc {
            Some(ref path) => {
                let mut f = OpenOptions::new().write(true).truncate(true).create(true).open(path.as_path())?;
                for ln in lines {
                    write!(f, "{}\n", ln)?;
                }
                f.sync_all()?;
                Ok(())
            },
            None => Err(IoError::new(ErrorKind::NotFound, "sync_disk with no file backing"))
        }
    }

    pub fn paint(&mut self, mut rx: &mut RenderContext, bnd: Rect) {
        //draw text
        let mut p = Point::xy(bnd.x, bnd.y);
        let mut line = self.viewport_start;
        while p.y < bnd.y+bnd.h && line < self.line_layouts.len() {
            let mut replace = false;
            match self.line_layouts[line] {
                Some(ref l) => { 
                    rx.draw_text_layout(p, &l, Color::rgb(0.9, 0.9, 0.9));
                    let b = l.bounds();
                    p.y += b.h;
                }
                None => {
                    replace = true; // a hacky way to get around the fact that the match borrows self.line_layouts,
                                    // so we can't assign to it until we escape the scope
                }
            }
            if replace {
                self.line_layouts[line] = TextLayout::new(&mut rx, &self.lines[line], &self.res.borrow().font,
                                                                  bnd.w, bnd.h).ok();
            } else {
                line += 1;
            }
        }
        self.viewport_end = line;

        //draw cursor
        if self.show_cursor {
            let col = self.cursor_col;
            let mut cb = self.line_layouts[self.cursor_line].as_ref().map_or(Rect::xywh(0.0, 0.0, 8.0, 8.0), |v| v.char_bounds(col));
            if cb.w == 0.0 { cb.w = 8.0; }
            rx.fill_rect(cb.offset(Point::xy(bnd.x,bnd.y+cb.h*(self.cursor_line.saturating_sub(self.viewport_start)) as f32)),
            Color::rgba(0.8, 0.6, 0.0, 0.9));
        }
    }

}
