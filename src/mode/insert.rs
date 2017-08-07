
use super::*;
use runic::{Event,KeyCode};
use Movement;

pub struct InsertMode;

impl Mode for InsertMode {
    fn event(&mut self, e: Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>> {
        match e {
            Event::Key(k,false) => {
                match k {
                    KeyCode::Character(c) => {
                        let loc = (bv.cursor_col, bv.cursor_line);
                        bv.buf.borrow_mut().insert_char(loc, c);
                        bv.cursor_col += 1;
                        bv.invalidate_line(loc.1);
                        None
                    }
                    KeyCode::Up => { bv.make_movement(Movement::Line(true)); None }
                    KeyCode::Down => { bv.make_movement(Movement::Line(false)); None }
                    KeyCode::Left => { bv.make_movement(Movement::Char(false)); None }
                    KeyCode::Right => { bv.make_movement(Movement::Char(true)); None }
                    KeyCode::Escape => { Some(Box::new(NormalMode::new())) }
                    _ => None
                }
            }
            _ => None
        }
    }

    fn status_tag(&self) -> &str { "INSERT" }
}
