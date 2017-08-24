
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
    Insert, InsertLine, Append, Command,
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
                    'o' => Some(Action::InsertLine),
                    ';' => Some(Action::Command),
                    ':' => Some(Action::Command),
                    'x' => Some(Action::Delete(Movement::Char(true))),
                    'd' => Movement::parse(s.split_at(i+1).1, false).map(Action::Delete),
                    'c' => Movement::parse(s.split_at(i+1).1, false).map(Action::Change),
                    'r' => cs.next().map(|(_,c)| Action::Replace(c)),
                    _ => Movement::parse(s, true).map(Action::Move),
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
    fn event(&mut self, e: Event, app: &mut app::State, win: WindowRef) -> Result<Option<Box<Mode>>, Box<Error>> {
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
                            app.mutate_buf(|b| b.make_movement(mv)); Ok(None)
                        },
                        Action::Delete(mv) => { app.mutate_buf(|b| b.delete_movement(mv)); Ok(None) },
                        Action::Change(mv) => {
                            app.mutate_buf(|b| b.delete_movement(mv)); 
                            Ok(Some(Box::new(InsertMode::new())))
                        },
                        Action::Insert => Ok(Some(Box::new(InsertMode::new()))),
                        Action::Command => Ok(Some(Box::new(CommandMode::new(app)))),
                        Action::Append => {
                            app.mutate_buf(|b| b.move_cursor((1,0)));
                            Ok(Some(Box::new(InsertMode::new())))
                        },
                        Action::InsertLine => {
                            app.mutate_buf(|b| { let loc = b.cursor_line; b.insert_line(loc) });
                            Ok(Some(Box::new(InsertMode::new())))
                        }
                        _ => { Ok(None) }
                    }
                } else { Ok(None) }
            },
            _ => { Ok(None) }
        }
    }
    fn status_tag(&self) -> &str { "NORMAL" }
    fn pending_command(&self) -> Option<&str> { if self.buf.len() > 0 { Some(&self.buf) } else { None } }
}


