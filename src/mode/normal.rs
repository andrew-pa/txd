
use super::*;
use winit::{WindowEvent};
use movement::Movement;
use app::ClipstackId;

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
    Delete(Movement, ClipstackId),
    Change(Movement, ClipstackId),
    Insert, InsertLine, Append, Command,
    Replace(char),
    Yank(Movement, ClipstackId),
    Put(ClipstackId, bool /* copy or pop */)
}

impl Action {
    fn parse(s: &str) -> Option<Action> {
        let mut cs = s.trim().char_indices().peekable();
        let mut reg = ClipstackId('"'); //default register is ""
        if let Some(&(_, '"')) = cs.peek() {
            if cs.next().is_none() { return None }
            reg = match cs.next() {
                Some((_, c)) => ClipstackId(c),
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
                    'p' => Some(Action::Put(reg, false)),
                    'P' => Some(Action::Put(reg, true)),
                    'r' => cs.next().map(|(_,c)| Action::Replace(c)),
                    _ => Movement::parse(s, true).map(Action::Move),
                }
            },
            None => None
        }
    }

    fn execute(&self, app: &mut app::State) -> Result<Option<Box<Mode>>, Box<Error>> {
        // could probably just take plain `self` and move into execute(), as it works in the only
        // context this function is called in right now
        match self {
            &Action::Move(ref mv) => {
                app.mutate_buf(|b| b.make_movement(mv.clone())); Ok(None)
            },
            &Action::Delete(ref mv, ref r) => {
                let v = app.mutate_buf(|b| b.delete_movement(mv.clone()));
                app.push_clip(r, v);
                Ok(None)
            },
            &Action::Change(ref mv, ref r) => {
                let v = app.mutate_buf(|b| b.delete_movement(mv.clone())); 
                app.push_clip(r, v);
                Ok(Some(Box::new(InsertMode::new())))
            },
            &Action::Replace(c) => app.mutate_buf(|b| {
                b.delete_char();
                b.insert_char(c);
                Ok(None)
            }),
            &Action::Insert => Ok(Some(Box::new(InsertMode::new()))),
            &Action::Command => Ok(Some(Box::new(CommandMode::new(app)))),
            &Action::Append => {
                app.mutate_buf(|b| b.move_cursor((1,0)));
                Ok(Some(Box::new(InsertMode::new())))
            },
            &Action::InsertLine => {
                app.mutate_buf(|b| { b.insert_line(None) });
                Ok(Some(Box::new(InsertMode::new())))
            },
            &Action::Yank(ref mv, ref r) => {
                let v = app.mutate_buf(|b| b.yank_movement(mv.clone()));
                app.push_clip(r, v);
                Ok(None)
            },
            &Action::Put(ref r, copy) => {
                let pv = if copy {
                    app.top_clip(r)
                } else {
                    app.pop_clip(r)
                };
                if pv.is_some() {
                    let mut _b = &mut app.bufs[app.current_buffer];
                    let mut b = _b.borrow_mut();
                    b.insert_string(&pv.unwrap());
                }
                Ok(None)
            },
            _ => { Ok(None) }
        }
    }
}

impl NormalMode {
    pub fn new() -> NormalMode {
        NormalMode { buf: String::new() }
    }
}

impl Mode for NormalMode {
    fn event(&mut self, e: WindowEvent, app: &mut app::State) -> Result<Option<Box<Mode>>, Box<Error>> {
        match e {
            WindowEvent::ReceivedCharacter(c) => {
                if !c.is_control() { self.buf.push(c); } 
                if let Some(a) = Action::parse(&self.buf) {
                    self.buf.clear();
                    a.execute(app)
                } else { Ok(None) }
            }
            WindowEvent::KeyboardInput { input: winit::KeyboardInput { virtual_keycode: Some(winit::VirtualKeyCode::Escape), .. }, .. } => {
                self.buf.clear(); Ok(None)
            }
            _ => { Ok(None) }
        }
    }
    fn status_tag(&self) -> &str { "NORMAL" }
    fn pending_command(&self) -> Option<&str> { if self.buf.len() > 0 { Some(&self.buf) } else { None } }
}


