
use runic::{App, Window as SystemWindow, Event, RenderContext, Color, Point, Rect, Font, TextLayout, KeyCode};
use std::rc::Rc;
use std::cell::RefCell;
use res::Resources;
use movement::Movement;
use buffer::Buffer;

pub struct BufferView {
    pub buf: Rc<RefCell<Buffer>>,
    pub res: Rc<RefCell<Resources>>,
    line_layouts: Vec<Option<TextLayout>>,
    viewport_start: usize,
    viewport_end: usize,
    pub cursor_line: usize,
    pub cursor_col: usize
}

impl BufferView {
    pub fn new(buf: Rc<RefCell<Buffer>>, mut rx: &mut RenderContext, res: Rc<RefCell<Resources>>) -> BufferView {
        let mut ed = BufferView {
            buf: buf, res: res,
            line_layouts: Vec::new(),
            viewport_start: 0, viewport_end: 0,
            cursor_line: 0, cursor_col: 0
        };
        let bnd = rx.bounds();
        for ref line in ed.buf.borrow().lines.iter() {
            ed.line_layouts.push(TextLayout::new(&mut rx, &line, &ed.res.borrow().font, bnd.w-4.0, bnd.h-4.0).ok());
        }
        ed
    }

    pub fn move_cursor(&mut self, (dx, dy): (isize,isize)) {
        let mut cursor_col = self.cursor_col as isize + dx;
        let mut cursor_line = self.cursor_line as isize + dy;
        if cursor_col < 0 { cursor_col = 0; }
        if cursor_line < 0 { cursor_line = 0; }
        let bl = &self.buf.borrow().lines;
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

    pub fn invalidate_line(&mut self, line: usize) {
        self.line_layouts[line] = None;
    }
    pub fn insert_line(&mut self, line: usize) {
        self.line_layouts.insert(line, None);
    }

    pub fn paint(&mut self, mut rx: &mut RenderContext, bnd: Rect) {
        //draw text
        let mut p = Point::xy(bnd.x, bnd.y);
        let mut line = self.viewport_start;
        while p.y < bnd.h && line < self.line_layouts.len() {
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
                self.line_layouts[line] = TextLayout::new(&mut rx, &self.buf.borrow().lines[line], &self.res.borrow().font,
                                                                  bnd.w, bnd.h).ok();
            } else {
                line += 1;
            }
        }
        self.viewport_end = line;

        //draw cursor
        let col = self.cursor_col;
        let mut cb = self.line_layouts[self.cursor_line].as_ref().map_or(Rect::xywh(0.0, 0.0, 8.0, 8.0), |v| v.char_bounds(col));
        if cb.w == 0.0 { cb.w = 8.0; }
        rx.fill_rect(cb.offset(Point::xy(bnd.x,bnd.y+cb.h*(self.cursor_line.saturating_sub(self.viewport_start)) as f32)),
            Color::rgba(0.8, 0.6, 0.0, 0.9));
    }
}



