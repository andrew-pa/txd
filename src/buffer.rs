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
        let mut f = OpenOptions::new().read(true).write(true).open(fp)?;
        let (lns, lay) = if fp_exists { 
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
        let cln = bl[cursor_line as usize].len();
        if cursor_col as usize > cln { cursor_col = cln as isize; }

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

    pub fn make_movement(&mut self, mv: Movement) {
        match mv {
            Movement::Rep(count, smv) => for _ in 0..count { self.make_movement(*smv.clone()); },
            Movement::Char(right) => self.move_cursor((if right {1} else {-1}, 0)),
            Movement::Line(up) => self.move_cursor((0, if up {-1} else {1})),
            _ => {}
        }
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
        self.line_layouts.insert(loc, None);
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
