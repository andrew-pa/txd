
use super::*;
use runic::{Event, KeyCode};
use Movement;

//Normal Mode
pub struct NormalMode {
    buf: String
}

// actions:
// [mov]: move cursor
// d[mov]: delete
// i: insert text
// c[mov]: change text
// r[char]: replace char
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
    pub fn new() -> NormalMode {
        NormalMode { buf: String::new() }
    }
}

impl Mode for NormalMode {
    fn event(&mut self, e: Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>> {
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


