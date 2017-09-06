
use super::*;
use runic::{Event, KeyCode, WindowRef};
use movement::Movement;
use app::RegisterId;

//Normal Mode
pub struct NormalMode {
    buf: String
}

// actions:
// [mov]: move cursor
// [reg]d[mov]: delete
// i: insert text
// [reg]c[mov]: change text
// r[char]: replace char
// [reg]y[mov]: yank (copy) text into reg
// [reg]p: put text out of reg
// reg: '"' followed with a register name (one char)
//    special registers:
//        "* => the system clipboard
//        "" => the register that gets the last yanked/deleted movement by default


#[derive(Debug)]
enum Action {
    Move(Movement),
    Delete(Movement, RegisterId),
    Change(Movement, RegisterId),
    Insert, InsertLine, Append, Command,
    Replace(char),
    Yank(Movement, RegisterId),
    Put(RegisterId)
}

impl Action {
    fn parse(s: &str) -> Option<Action> {
        let mut cs = s.trim().char_indices().peekable();
        let mut reg = RegisterId('"'); //default register is ""
        if let Some(&(_, '"')) = cs.peek() {
            if cs.next().is_none() { return None }
            reg = match cs.next() {
                Some((_, c)) => RegisterId(c),
                None => return None
            };
        }
        match cs.next() {
            Some((i, c)) => {
                //println!("i,c {} {}", i, c);
                match c {
                    'i' => Some(Action::Insert),
                    'a' => Some(Action::Append),
                    'o' => Some(Action::InsertLine),
                    ';' => Some(Action::Command),
                    ':' => Some(Action::Command),
                    'x' => Some(Action::Delete(Movement::Char(true), reg)),
                    'd' => Movement::parse(s.split_at(i+1).1, false).map(|m| Action::Delete(m,reg)),
                    'c' => Movement::parse(s.split_at(i+1).1, false).map(|m| Action::Change(m,reg)),
                    'y' => Movement::parse(s.split_at(i+1).1, false).map(|m| Action::Yank(m,reg)),
                    'p' => Some(Action::Put(reg)),
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
                    KeyCode::Character(c) => { if !c.is_control() { self.buf.push(c); } }
                    KeyCode::Escape => { self.buf.clear(); }
                    _ => { }
                }
                if let Some(a) = Action::parse(&self.buf) {
                    self.buf.clear();
                    match a {
                        Action::Move(mv) => {
                            app.mutate_buf(|b| b.make_movement(mv)); Ok(None)
                        },
                        Action::Delete(mv, r) => { let v = app.mutate_buf(|b| b.delete_movement(mv)); app.registers.insert(r, v); Ok(None) },
                        Action::Change(mv, r) => {
                            let v = app.mutate_buf(|b| b.delete_movement(mv)); 
                            app.registers.insert(r, v);
                            Ok(Some(Box::new(InsertMode::new())))
                        },
                        Action::Replace(c) => app.mutate_buf(|b| {
                            b.delete_char();
                            b.insert_char(c);
                            Ok(None)
                        }),
                        Action::Insert => Ok(Some(Box::new(InsertMode::new()))),
                        Action::Command => Ok(Some(Box::new(CommandMode::new(app)))),
                        Action::Append => {
                            app.mutate_buf(|b| b.move_cursor((1,0)));
                            Ok(Some(Box::new(InsertMode::new())))
                        },
                        Action::InsertLine => {
                            app.mutate_buf(|b| { b.insert_line(None) });
                            Ok(Some(Box::new(InsertMode::new())))
                        },
                        Action::Yank(mv, r) => {
                            let v = { app.bufs[app.current_buffer].borrow().yank_movement(mv) };
                            app.registers.insert(r, v);
                            Ok(None)
                        },
                        Action::Put(r) => {
                            let pv = app.registers.get(&r);
                            if pv.is_some() {
                                let mut _b = &mut app.bufs[app.current_buffer];
                                let mut b = _b.borrow_mut();
                                b.insert_string(pv.unwrap());
                            }
                            Ok(None)
                        },
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


