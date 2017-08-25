
use super::*;
use runic::{Event,KeyCode,WindowRef};
use movement::Movement;

pub struct InsertMode {
    target_buffer: Option<usize>
}

impl InsertMode {
    pub fn new() -> InsertMode { InsertMode { target_buffer: None } }
    pub fn new_with_target(target: usize) -> InsertMode { InsertMode { target_buffer: Some(target) } }
}

impl Mode for InsertMode {
    fn event(&mut self, e: Event, app: &mut app::State, _: WindowRef) -> Result<Option<Box<Mode>>, Box<Error>> {
        let mut buf_ = match self.target_buffer {
            Some(target) => app.bufs[target].clone(),
            None => app.buf(),
        };
        let mut buf = buf_.borrow_mut();
        let cloc = buf.curr_loc();
        match e {
            Event::Key(k,true) => {
                match k {
                    KeyCode::Enter => {
                        buf.break_line(cloc);
                        Ok(None)
                    }
                    KeyCode::Delete => {
                        buf.delete_char(cloc);
                        Ok(None)
                    }
                    KeyCode::Backspace => {
                        if cloc.0 != 0 {
                            buf.move_cursor((-1, 0));
                            buf.delete_char((cloc.0-1, cloc.1));
                        }
                        Ok(None)
                    }
                    KeyCode::Character(c) => {
                        if c.is_control() { Ok(None) } else {
                            buf.insert_char(cloc, c);
                            buf.move_cursor((1, 0));
                            Ok(None)
                        }
                    }
                    KeyCode::Up => { buf.make_movement(Movement::Line(true)); Ok(None) }
                    KeyCode::Down => { buf.make_movement(Movement::Line(false)); Ok(None) }
                    KeyCode::Left => { buf.make_movement(Movement::Char(false)); Ok(None) }
                    KeyCode::Right => { buf.make_movement(Movement::Char(true)); Ok(None) }
                    KeyCode::Escape => { Ok(Some(Box::new(NormalMode::new()))) }
                    _ => Ok(None)
                }
            }
            _ => Ok(None)
        }
    }

    fn status_tag(&self) -> &str { "INSERT" }
}
