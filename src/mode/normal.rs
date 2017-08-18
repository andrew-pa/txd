
use super::*;
use runic::{Event, KeyCode, WindowRef};
use movement::Movement;

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
    Insert, Append, Command,
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
                    'a' => Some(Action::Append),
                    ';' => Some(Action::Command),
                    ':' => Some(Action::Command),
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
    fn event(&mut self, e: Event, app: &mut app::State, win: WindowRef) -> Option<Box<Mode>> {
        match e {
            Event::Key(k, d) => {
                match k {
                    KeyCode::Character(c) => { self.buf.push(c); }
                    KeyCode::Escape => { self.buf.clear(); }
                    _ => { }
                }
                if let Some(a) = Action::parse(&self.buf) {
                    self.buf.clear();
                    match a {
                        Action::Move(mv) => {
                            app.ed.make_movement(mv); None
                        },
                        Action::Insert => Some(Box::new(InsertMode)),
                        Action::Command => Some(Box::new(CommandMode::new())),
                        Action::Append => {
                            app.ed.move_cursor((1,0));
                            Some(Box::new(InsertMode))
                        }
                        _ => { None }
                    }
                } else { None }
            },
            _ => { None }
        }
    }
    fn status_tag(&self) -> &str { "NORMAL" }
    fn pending_command(&self) -> Option<&str> { if self.buf.len() > 0 { Some(&self.buf) } else { None } }
}


