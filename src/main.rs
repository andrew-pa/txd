extern crate runic;

use runic::{App, Window as SystemWindow, Event, RenderContext, Color, Point, Rect, Font, TextLayout, KeyCode};
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

mod buffer;
use buffer::Buffer;
use std::path::Path;

// txd: a text editor🖳


/* TODO
 * + Get basic modal text editing working
 *   + Status line
 *   + Command line
 *     + Good command parsing
 *   + Get resonable UX together (ie not opening src\main.rs at load)
 * + Make buffer rep more reasonable
 * + Configuration stuff (colors! fonts! commands?)
 * + Language Server Protocol
 *   + low-level client
 *   + callbacks/tie-ins
 *   + syntax highlighting!
 *   + ensure it works/can be configured right with several different servers
 */

pub struct Resources {
    font: Font
}

impl Resources {
    fn new(mut rx: &mut RenderContext) -> Result<Resources, Box<Error>> {
        Ok(Resources {
            font: Font::new(&mut rx, String::from("Consolas"), 16.0, runic::FontWeight::Regular, runic::FontStyle::Normal)?,
        })
    }
}

mod bufferview;
use bufferview::BufferView;


struct TxdApp {
    buf: Rc<RefCell<Buffer>>,
    res: Rc<RefCell<Resources>>,
    ed: BufferView
}

impl TxdApp {
    fn init(mut rx: &mut RenderContext) -> TxdApp {
        let buf = Rc::new(RefCell::new(Buffer::load(Path::new("src\\main.rs")).expect("open file")));
        let res = Rc::new(RefCell::new(Resources::new(rx).expect("create resources")));
        let ed = BufferView::new(buf.clone(), &mut rx, res.clone());
        TxdApp { buf, res, ed }
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event) {
        match e {
            Event::Key(k, d) => match k {
                KeyCode::Character('h') => { self.ed.move_cursor((-1, 0)); }
                KeyCode::Character('j') => { self.ed.move_cursor((0, 1)); }
                KeyCode::Character('k') => { self.ed.move_cursor((0, -1)); }
                KeyCode::Character('l') => { self.ed.move_cursor((1, 0)); }
                KeyCode::Character('x') => {
                    let (co,li) = (self.ed.cursor_col, self.ed.cursor_line);
                    self.buf.borrow_mut().lines[li].remove(co);
                    self.ed.line_layouts[li] = None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn paint(&mut self, mut rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        let bnd = rx.bounds();
        self.ed.paint(rx, Rect::xywh(4.0, 4.0, bnd.w-4.0, bnd.h-14.0));
    }
}

fn main() {
    SystemWindow::new("txd", 1280, 640, TxdApp::init).expect("create window!").show();
}
