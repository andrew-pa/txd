use std::rc::Rc;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::fs::*;
use std::io::{Read, Write, Error as IoError, ErrorKind};
use std::cmp::min;

use runic::*;
use res::Resources;
use movement::*;

use std::str::*;

#[derive(Debug)]
pub enum TabStyle {
    Tab,
    Spaces(usize)
}


#[derive(Debug)]
pub enum Yank {
    Span(String), // a span of characters that are intra-line
    Lines(Vec<String>) // one or more lines of text
}

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
    pub show_cursor: bool,

    pub tab_style: TabStyle,
}

impl Buffer {
    pub fn new(res: Rc<RefCell<Resources>>) -> Buffer {
        Buffer {
            fs_loc: None, lines: vec![String::from("")],
            res, cursor_line: 0, cursor_col: 0, viewport_start: 0, viewport_end: 0,
            line_layouts: vec![None], show_cursor: true, tab_style: /* should be config */ TabStyle::Tab
        }
    }

    pub fn load(fp: &Path, res: Rc<RefCell<Resources>>) -> Result<Buffer, IoError> {
        let fp_exists = fp.exists();
        
        let (lns, lay, ts) = if fp_exists { 
            let mut f = OpenOptions::new().read(true).write(true).open(fp)?;
            let mut s : String = String::new();
            f.read_to_string(&mut s)?;
            let lns: Vec<String> = s.lines().map(String::from).collect();
            let mut layouts = Vec::new();
            let mut ts: Option<TabStyle> = None;
            for i in 0..lns.len() { //replace with Vec::resize_default?
                if ts.is_none() {
                    let mut ch = lns[i].chars();
                    ts = match ch.next() {
                        Some('\t') => Some(TabStyle::Tab),
                        Some(' ') => {
                            let mut n = 1;
                            while let Some(' ') = ch.next() { n += 1 }
                            Some(TabStyle::Spaces(n))
                        },
                        _ => None
                    };
                }
                layouts.push(None);
            }
            //println!("detected tab style = {:?}", ts);
            (lns, layouts, ts.unwrap_or(TabStyle::Tab /* config details */))
        } else {
            (vec![String::from("")], vec![None], /* should be config just like ::new */ TabStyle::Tab)
        };
        let mut buf = Buffer {
            fs_loc: Some(PathBuf::from(fp)),
            lines: lns, line_layouts: lay, viewport_start: 0, viewport_end: 0, cursor_line: 0, cursor_col: 0, show_cursor: true, res,
            tab_style: ts
        };
        Ok(buf)
    }

    pub fn place_cursor(&mut self, mut cursor_col: usize, mut cursor_line: usize) {
        let bl = &self.lines;
        if bl.len() == 0 { cursor_line = 0; }
        else {
            if cursor_line >= bl.len() { cursor_line = (bl.len()-1); } 

            let cln = &bl[cursor_line];
            if cursor_col > cln.len() { cursor_col = cln.len(); }
            /*let mut lowest_dist = 1000;
            for (i,g) in &cln[..].grapheme_indices() {
                let dist = (cursor_col - i).abs();
                if dist < lowest_dist {
                    lowest_dist = dist;
                    cursor_col = i;
                }
            }*/
            while !cln.is_char_boundary(cursor_col) { println!("{}", cursor_col); cursor_col += 1; }
        }


        while cursor_line < self.viewport_start {
            self.viewport_start = self.viewport_start.saturating_sub(1);
        }
        while cursor_line >= self.viewport_end {
            let len = self.viewport_end - self.viewport_start;
            self.viewport_start = self.viewport_start.saturating_add(1);
            self.viewport_end = self.viewport_start+len;
        }

        self.cursor_col = cursor_col;
        self.cursor_line = cursor_line;
    }

    pub fn move_cursor(&mut self, (dx, dy): (isize,isize)) {
        let mut cursor_col = self.cursor_col as isize + dx;
        let mut cursor_line = self.cursor_line as isize + dy;
        if cursor_col < 0 { cursor_col = 0; }
        if cursor_line < 0 { cursor_line = 0; }
        self.place_cursor(cursor_col as usize, cursor_line as usize);
   }

    pub fn curr_loc(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_line)
    }

    // scan from cursor looking for character. possibly absurdly made and could be done better with
    // a better buffer representation
    pub fn scan_line<P: Fn(char)->bool>(&self, pred: P, forwards: bool) -> Option<usize> {
        let mut line_chars = self.lines[self.cursor_line].char_indices();
        (if forwards {
            println!("fwd");
            for (i, c) in line_chars {
                if i < self.cursor_col { continue }
                println!("{:?}", (i,c));
                if(pred(c)) { return Some(i); }
            }
            None //line_chars.take(self.cursor_col).inspect(|&v| print!("{:?}", v)).find(|&(_, c)| pred(c)).map(|(i, _)| i)
        } else {
            println!("rev");
            for (i, c) in line_chars.rev() {
                if i > self.cursor_col { continue }
                println!("{:?}", (i,c));
                if(pred(c)) { return Some(i); }
            }
            None
            //line_chars.skip(self.cursor_col).inspect(|&v| print!("{:?}", v)).find(|&(_, c)| pred(c)).map(|(i, _)| i)
        })
        /*let (left, right) = self.lines[self.cursor_line].split_at(self.cursor_col + if forwards {1} else {0});
        //println!("({}, {})", left, right);
        if forwards {
            right.find(pred).map(|v| v as isize + 1)
        } else {
            left.rfind(pred).map(|v| -(left.len() as isize - v as isize))
        }*/
    }


    /// calculate the range of a movement from where the cursor is currently located to the end of
    /// the movement, in the range start <= x < end, like (start..end). The first tuple will usually be the same as the current cursor
    /// location except in cases where the movement includes an entire line, for instance. The
    /// second tuple is the end of the movement absolute.
    pub fn movement_range(&mut self, mv: &Movement) -> ::std::ops::Range<(usize, usize)> {
        fn wrapadd1(a: usize, b: bool) -> usize {
            if b { a.saturating_add(1) } else { a.saturating_sub(1) }
        }
        //println!("movement = {:?}", mv);
        let cur = self.curr_loc();
        match *mv {
            Movement::Char(right) => (cur..(wrapadd1(cur.0, right), cur.1)),
            Movement::Line(up, m) => match m {
                Inclusion::Linewise => (cur..(cur.0, wrapadd1(cur.1, !up))),
                Inclusion::Inclusive => ((0, cur.1)..(0, wrapadd1(cur.1, !up))),
                Inclusion::Exclusive => panic!("Exclusive line movement")
            },
            Movement::CharScan { query, direction, inclusion, place_to_side } => {
                match self.scan_line(|q| q==query, direction) {
                    Some(col) => { (cur..(if place_to_side { wrapadd1(col, !direction) } else { col }, cur.1)) },
                    None => (cur..cur)
                }
            }
            Movement::Word(direction, inclusion) => {
                /*
                 * if we're on whitespace => move to next non-whitespace character
                 * if we're on alphanumun => move until we hit non-alphanum character
                 *                           if whitespace than move one past that
                 * if we're on non-alphanum => move until we hit alphanum
                */
                let mut v = (cur..cur);
                'main: while v.end.1 < self.lines.len() {
                    let mut chars: Box<Iterator<Item = (usize, char)>> = if direction {
                        Box::new(self.lines[v.end.1].char_indices().skip(v.end.0)) 
                    } else {
                        Box::new(self.lines[v.end.1].char_indices().rev().skip(self.lines[v.end.1].len()-v.end.0))
                    };
                    let mut j = 0;
                    match chars.next() {
                        Some((i ,c)) => {
                            if char::is_alphanumeric(c) {
                                for (i, c) in chars {
                                    if !char::is_alphanumeric(c) {
                                        v.end = (i + if !char::is_whitespace(c) { 0 } else { 1 }, v.end.1);
                                        break 'main;
                                    }
                                }
                            } else if char::is_whitespace(c) { 
                                for (i, c) in chars {
                                    if !char::is_whitespace(c) { v.end = (i, v.end.1); break 'main; }
                                }
                            } else {
                                for (i, c) in chars {
                                    if char::is_alphanumeric(c) { v.end = (i, v.end.1); break 'main; }
                                }
                            }
                        }
                        None => {
                        }
                    }
                    let y = wrapadd1(v.end.1, direction);
                    v.end = (if direction { 0 } else { self.lines[y].len() }, y);
                }
                v
            },
            Movement::EndOfLine => (cur..(self.lines[self.cursor_line].len()-1, cur.1)),
            Movement::StartOfLine => (cur..(0,cur.1)),
            Movement::Rep(count, ref movement) => {
                let mut total_range = self.movement_range(movement);
                let cp = self.curr_loc();
                for _ in 1..count {
                    self.place_cursor(total_range.end.0, total_range.end.1);
                    let r = self.movement_range(movement);
                    if r.start.1 < total_range.start.1 || r.start.0 < total_range.start.0 {
                        total_range.start = r.start;
                    }
                    if r.end.1 > total_range.end.1 || r.end.0 > total_range.end.0 {
                        total_range.end = r.end;
                    }
                }
                self.place_cursor(cp.0, cp.1);
                total_range
            }
            _ => panic!("unknown movement!")
        }
    }

    pub fn make_movement(&mut self, mv: Movement) {
        let new_pos = self.movement_range(&mv).end;
        self.place_cursor(new_pos.0, new_pos.1);
    }

    pub fn delete_movement(&mut self, mv: Movement) -> String {
        let mut removed = String::new();
        // movement_range(mv) calculates range for movement mv. Must delete all selected
        // lines+chars. Easy ranges are say (6, n) -> (10, n) where it's all in one line. the
        // harder ranges are those like (0, n) -> (0, n+3) where it deletes whole lines, and the
        // hardest are probably ones like (6,7) -> (8,12) where it deletes whole lines and
        // intraline characters
        println!("trying to delete movement ({:?})", mv);
        let incm = mv.inclusion_mode();
        let ::std::ops::Range { mut start, mut end } = self.movement_range(&mv);
        println!("\tfrom {:?} to {:?}", start, end);
        self.line_layouts[start.1] = None;
        for line in (start.1)..(end.1) {
            println!("\tline {}: {}", line, self.lines[line]);
            self.line_layouts[line] = None;
        }

        if incm == Inclusion::Inclusive { end.0 += 1; }

        if start.1 == end.1 { // all in the same line
            removed.push_str(&self.lines[start.1]
                           .drain(if start.0 > end.0 { (end.0)..(start.0) } else { (start.0)..(end.0) })
                           .collect::<String>());
        } else {
            for i in (start.1)..(end.1) {
                removed.push_str(&self.lines.remove(i));
                removed.push_str("\n");
                self.line_layouts.remove(i);
            }
        }
        if self.lines.len() == 0 {
            self.lines.push(String::new());
            self.line_layouts.push(None);
        }
        println!("\t removed: \"{}\"", removed);
        self.move_cursor((0,0));  //ensure that the cursor is in a valid position
        removed
    }

    pub fn yank_movement(&mut self, mv: Movement) -> String {
        let mut selected = String::new();
        println!("trying to yank movement ({:?})", mv);
        let incm = mv.inclusion_mode();
        let ::std::ops::Range { mut start, mut end } = self.movement_range(&mv);
        println!("\tfrom {:?} to {:?}", start, end);
        for line in (start.1)..(end.1) {
            println!("\tline {}: {}", line, self.lines[line]);
        }

        if incm == Inclusion::Inclusive { end.0 += 1; }

        if start.1 == end.1 { // all in the same line
            selected.push_str(&self.lines[start.1][if start.0 > end.0 { (end.0)..(start.0) } else { (start.0)..(end.0) }]);
        } else {
            for i in (start.1)..(end.1) {
                selected.push_str(&self.lines[i]);
                selected.push_str("\n");
            }
        }
        
        println!("\t yanked: \"{}\"", selected);
        selected
        /*match mv {
            Movement::WholeLine => {
                self.lines[self.cursor_line].clone() + "\n"
            },
            Movement::Rep(count, box Movement::WholeLine) => {
                self.lines.iter().skip(self.cursor_line).take(count)
                    .fold(String::from(""), |s,l| s+&l+"\n")
            },
            _ => {
                let offset = self.movement_cursor_offset(mv);
                if offset.1 == 0 { //deleting within the current line
                    let last = min((offset.0 + self.cursor_col as isize) as usize, self.lines[self.cursor_line].len());
                    //println!("yanking: {}, {}", self.cursor_col, last);
                    let mut s = String::new();
                    s.push_str(&self.lines[self.cursor_line][if self.cursor_col > last { last..self.cursor_col } else { self.cursor_col..last }]);
                    s
                } else {
                    panic!("tried to yank multiline range");
                }
            }
        }*/
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

    pub fn insert_char(&mut self, c: char) {
        let loc = self.curr_loc();
        self.lines[loc.1].insert(loc.0, c);
        self.invalidate_line(loc.1);
        self.move_cursor((1, 0));
    }
    pub fn delete_char(&mut self) {
        let loc = self.curr_loc();
        if loc.0 >= self.lines[loc.1].len() {
            self.lines[loc.1].pop();
        }
        else {
            self.lines[loc.1].remove(loc.0);
        }
        self.invalidate_line(loc.1);
    }
    pub fn break_line(&mut self) {
        let loc = self.curr_loc();
        let new_line = if loc.0 >= self.lines[loc.1].len() {
            String::from("")
        } else {
            self.lines[loc.1].split_off(loc.0)
        };
        self.lines.insert(loc.1+1, new_line);
        self.invalidate_line(loc.1);
        self.line_layouts.insert(loc.1, None);
        self.viewport_end += 1;
        self.cursor_col = 0; self.move_cursor((0,1));
    }
    pub fn insert_line(&mut self, val: Option<&str>) {
        let loc = self.cursor_line;
        self.lines.insert(loc+1, val.map(|s| String::from(s)).unwrap_or_default());
        self.line_layouts.insert(loc+1, None);
        self.viewport_end += 1;
        self.cursor_col = 0; self.move_cursor((0,1));
    }
    pub fn insert_tab(&mut self) {
        match self.tab_style {
            TabStyle::Spaces(num) => {
                for _ in 0..num { self.insert_char(' '); }
            },
            TabStyle::Tab => {
                self.insert_char('\t');
                //sketchy cursor moves one tab at a time ⇒ can't break tabs in the middle. Why would anyone do that anyways...
            }
        }
    }
    pub fn insert_string(&mut self, s: &String) {
        // if the string has '\n' at the end → then insert it on it's own new line
        // else → start inserting in the middle of the current line
        if s.as_bytes()[s.len()-1] == b'\n' {
            for ln in s.lines() {
                self.insert_line(Some(ln));
            }
        } else {
            let mut lns = s.lines();
            let fln = lns.next().unwrap();
            let loc = self.curr_loc();
            self.lines[loc.1].insert_str(loc.0, fln);
            self.invalidate_line(loc.1);
            self.move_cursor((fln.len() as isize, 0));
            for ln in lns {
                self.insert_line(Some(ln));
            }
        }
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
        rx.set_color(Color::rgb(0.9, 0.9, 0.9));
        'lineloop: while line < self.line_layouts.len() {
            let mut replace = false;
            match self.line_layouts[line] {
                Some(ref l) => { 
                    let b = l.bounds();
                    if p.y + b.h > bnd.y+bnd.h { break 'lineloop; }
                    rx.draw_text_layout(p, &l);

                    //draw cursor
                    if self.show_cursor && line == self.cursor_line {
                        let col = self.cursor_col;
                        let mut cb = self.line_layouts[self.cursor_line].as_ref().map_or(Rect::xywh(0.0, 0.0, 8.0, 8.0), |v| v.char_bounds(col));
                        if cb.w == 0.0 { cb.w = 8.0; }
                        rx.set_color(Color::rgba(0.8, 0.6, 0.0, 0.9));
                        rx.fill_rect(cb.offset(p));
                        rx.set_color(Color::rgb(0.9, 0.9, 0.9));
                    }

                    p.y += b.h;
                }
                None => {
                    replace = true; // a hacky way to get around the fact that the match borrows self.line_layouts,
                    // so we can't assign to it until we escape the scope
                }
            }
            if replace {
                self.line_layouts[line] = rx.new_text_layout(&self.lines[line], &self.res.borrow().font,
                bnd.w, bnd.h).ok();
            } else {
                line += 1;
            }
        }
        self.viewport_end = line;
    }

    pub fn move_cursor_to_mouse(&mut self, p: Point) {
    }
}
