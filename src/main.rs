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
            font: Font::new(&mut rx, "Consolas", 16.0, runic::FontWeight::Regular, runic::FontStyle::Normal)?,
        })
    }
}

mod bufferview;
use bufferview::BufferView;

trait Mode {
    fn event(&mut self, e: Event, bv: &mut BufferView) -> Option<Box<Mode>>;
    fn status_tag(&self) -> &str;
}

// Normal Mode 
// movements:
// hjkl: ±1 char/line
// w: forward one word
// b: backward one word
// e: forward one word, place at end
// <number>[mov]: repeated movement n times
// actions:
// [mov]: move cursor
// d[mov]: delete
// i: insert text
// c[mov]: change text
// r[char]: replace char
struct NormalMode {
    buf: String
}

#[derive(Debug, Clone)]
enum Movement {
    Char(bool),
    Line(bool),
    Word(bool, bool),
    Rep(usize, Box<Movement>)
}

impl Movement {
    fn parse(s: &str) -> Option<Movement> {
        //println!("parse movment {}", s);
        use Movement::*;
        let mut cs = s.char_indices();
        match cs.next() {
            Some((i, c)) => {
                if c.is_digit(10) {
                    let start = i; let mut end = 0;
                    for (j,c) in cs {
                        if !c.is_digit(10) { end = j; break; }
                    }
                    let sp = s.split_at(end);
                    sp.0.parse::<usize>().ok().and_then(|n| Movement::parse(sp.1).map(|m| Rep(n,Box::new(m)))) 
                } else {
                    match c {
                        'h' => Some(Char(false)),
                        'j' => Some(Line(false)),
                        'k' => Some(Line(true)),
                        'l' => Some(Char(true)),
                        'w' => Some(Word(false, false)),
                        'b' => Some(Word(true, false)),
                        'e' => Some(Word(false, true)),
                        _ => None
                    }
                }
            },
            None => None
        }
    }
}


#[derive(Debug)]
enum Action {
    Move(Movement),
    Delete(Movement),
    Change(Movement),
    Insert,
    Replace(char)
}

impl Action {
    fn parse(s: &str) -> Option<Action> {
        let mut cs = s.char_indices();
        match cs.next() {
            Some((i, c)) => {
                //println!("i,c {} {}", i, c);
                match c {
                    'i' => Some(Action::Insert),
                    'd' => Movement::parse(s.split_at(i+1).1).map(Action::Delete),
                    'c' => Movement::parse(s.split_at(i+1).1).map(Action::Change),
                    'r' => cs.next().map(|(_,c)| Action::Replace(c)),
                    _ => Movement::parse(s).map(Action::Move),
                }
            },
            None => None
        }
    }
}

impl NormalMode {
    fn new() -> NormalMode {
        NormalMode { buf: String::new() }
    }
}

impl Mode for NormalMode {
    fn event(&mut self, e: Event, bv: &mut BufferView) -> Option<Box<Mode>> {
        match e {
            Event::Key(k, d) => {
                match k {
                    /*KeyCode::Character('h') => { bv.move_cursor((-1, 0)); None }
                      KeyCode::Character('j') => { bv.move_cursor((0, 1)); None }
                      KeyCode::Character('k') => { bv.move_cursor((0, -1)); None }
                      KeyCode::Character('l') => { bv.move_cursor((1, 0)); None }
                      KeyCode::Character('x') => {
                      let (co,li) = (bv.cursor_col, bv.cursor_line);
                      bv.buf.borrow_mut().lines[li].remove(co);
                      bv.invalidate_line(li);
                      None
                      }*/
                    KeyCode::Character(c) => { self.buf.push(c); }
                    KeyCode::Escape => { self.buf.clear(); }
                    _ => { }
                }
                if let Some(a) = Action::parse(&self.buf) {
                    self.buf.clear();
                    match a {
                        Action::Move(mv) => {
                            bv.make_movement(mv); None
                        },
                        _ => { None }
                    }
                } else { None }
            },
            _ => { None }
        }
    }
    fn status_tag(&self) -> &str { "NORMAL" }
}

struct TxdApp {
    buf: Rc<RefCell<Buffer>>,
    res: Rc<RefCell<Resources>>,
    ed: BufferView,
    mode: Box<Mode>
}

impl TxdApp {
    fn init(mut rx: &mut RenderContext) -> TxdApp {
        let buf = Rc::new(RefCell::new(Buffer::load(Path::new("src\\main.rs")).expect("open file")));
        let res = Rc::new(RefCell::new(Resources::new(rx).expect("create resources")));
        let ed = BufferView::new(buf.clone(), &mut rx, res.clone());
        TxdApp { buf, res, ed, mode: Box::new(NormalMode::new()) }
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event) {
        self.mode.event(e, &mut self.ed);
    }

    fn paint(&mut self, mut rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        let bnd = rx.bounds();
        self.ed.paint(rx, Rect::xywh(4.0, 4.0, bnd.w-4.0, bnd.h-34.0));
        rx.fill_rect(Rect::xywh(0.0, bnd.h-34.0, bnd.w, 18.0), Color::rgb(0.25, 0.22, 0.2));
    }
}

fn main() {
    SystemWindow::new("txd", 1280, 640, TxdApp::init).expect("create window!").show();
}
