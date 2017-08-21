
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
    fn event(&mut self, e: Event, app: &mut app::State, _: WindowRef) -> Option<Box<Mode>> {
        let mut buf_ = match self.target_buffer {
            Some(target) => app.bufs[target].clone(),
            None => app.buf(),
        };
        let mut buf = buf_.borrow_mut();
        let cloc = buf.curr_loc();
        match e {
            Event::Key(k,false) => {
                match k {
                    KeyCode::Enter => {
                        buf.break_line(cloc);
                        None
                    }
                    KeyCode::Delete => {
                        buf.delete_char(cloc);
                        None
                    }
                    KeyCode::Backspace => {
                        buf.delete_char(cloc);
                        buf.move_cursor((-1, 0));
                        None
                    }
                    KeyCode::Character(c) => {
                        if c.is_control() { None } else {
                            buf.insert_char(cloc, c);
                            buf.move_cursor((1, 0));
                            None
                        }
                    }
                    KeyCode::Up => { buf.make_movement(Movement::Line(true)); None }
                    KeyCode::Down => { buf.make_movement(Movement::Line(false)); None }
                    KeyCode::Left => { buf.make_movement(Movement::Char(false)); None }
                    KeyCode::Right => { buf.make_movement(Movement::Char(true)); None }
                    KeyCode::Escape => { Some(Box::new(NormalMode::new())) }
                    _ => None
                }
            }
            _ => None
        }
    }

    fn status_tag(&self) -> &str { "INSERT" }
}
