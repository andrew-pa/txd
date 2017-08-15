
use super::*;
use runic::{Event,KeyCode};
use movement::Movement;

pub struct InsertMode;

impl Mode for InsertMode {
    fn event(&mut self, e: Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>> {
        match e {
            Event::Key(k,false) => {
                match k {
                    KeyCode::Enter => {
                        let loc = bv.curr_loc(); 
                        bv.buf.borrow_mut().break_line(loc);
                        bv.invalidate_line(loc.1);
                        bv.insert_line(loc.1);
                        bv.cursor_col = 0;
                        bv.move_cursor((0, 1));
                        None
                    }
                    KeyCode::Delete => {
                        let loc = bv.curr_loc();
                        bv.buf.borrow_mut().delete_char(loc);
                        bv.invalidate_line(loc.1);
                        None
                    }
                    KeyCode::Backspace => {
                        let loc = bv.curr_loc();
                        bv.buf.borrow_mut().delete_char(loc);
                        bv.move_cursor((-1, 0));
                        bv.invalidate_line(loc.1);
                        None
                    }
                    KeyCode::Character(c) => {
                        if c.is_control() { None } else {
                        let loc = bv.curr_loc();
                        bv.buf.borrow_mut().insert_char(loc, c);
                        bv.move_cursor((1, 0));
                        bv.invalidate_line(loc.1);
                        None
                        }
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
