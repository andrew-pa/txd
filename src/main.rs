extern crate runic;

use runic::{App, Window as SystemWindow, Event, RenderContext, Color, Point, Rect, Font, TextLayout, KeyCode};
use std::error::Error;

mod buffer;
use buffer::Buffer;
use std::path::Path;

// txd: a text editor🖳


struct TxdApp {
    buf: Buffer,
    fnt: Font,
    line_layouts: Vec<Option<TextLayout>>,
    cursor_line: usize,
    cursor_col: usize
}

impl TxdApp {
    fn init(mut rx: &mut RenderContext) -> TxdApp {
        let mut app = TxdApp {
            buf: Buffer::load(Path::new("src\\main.rs")).expect("open file"),
            fnt: Font::new(&mut rx, String::from("Consolas"), 14.0, runic::FontWeight::Regular, runic::FontStyle::Normal).expect("font"),
            line_layouts: Vec::new(),
            cursor_line: 0, cursor_col: 1
        };
        let bnd = rx.bounds();
        for ref line in app.buf.lines.iter() {
            app.line_layouts.push(TextLayout::new(&mut rx, &line, &app.fnt, bnd.w-4.0, bnd.h-4.0).ok());
        }
        app
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event) {
        match e {
            Event::Key(k, d) => match k {
                KeyCode::Character('h') => { self.cursor_col -= 1; }
                KeyCode::Character('j') => { self.cursor_line += 1; }
                KeyCode::Character('k') => { self.cursor_line -= 1; }
                KeyCode::Character('l') => { self.cursor_col += 1; }
                KeyCode::Character('x') => {
                    let (co,li) = (self.cursor_col, self.cursor_line);
                    self.buf.lines[li].remove(co);
                    self.line_layouts[li] = None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn paint(&mut self, mut rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        let mut p = Point::xy(4.0, 4.0);
        let bnd = rx.bounds();
        let mut line = 0;
        while p.y < bnd.h && line < self.line_layouts.len() {
            let mut replace = false;
            match self.line_layouts[line] {
                Some(ref l) => { 
                    rx.draw_text_layout(p, &l, Color::rgb(0.9, 0.9, 0.9));
                    let b = l.bounds();
                    p.y += b.h;
                }
                None => {
                    replace = true;
                }
            }
            if replace { self.line_layouts[line] = TextLayout::new(&mut rx, &self.buf.lines[line], &self.fnt,
                                                                   bnd.w-4.0, bnd.h-4.0).ok(); } else {
            line += 1;
            }
        }
        let col = self.cursor_col;
        let mut cb = self.line_layouts[self.cursor_line].as_ref().unwrap().char_bounds(col);
        if cb.w == 0.0 { cb.w = 8.0; }
        rx.fill_rect(cb.offset(Point::xy(4.0,4.0+cb.h*self.cursor_line as f32)), Color::rgb(0.8, 0.6, 0.0));
    }
}

fn main() {
    SystemWindow::new("txd", 1280, 640, TxdApp::init).expect("create window!").show();
}
