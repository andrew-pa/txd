
use super::*;
use runic::{Event,KeyCode};
use movement::Movement;

pub struct InsertMode;

impl Mode for InsertMode {
    fn event(&mut self, e: Event, app: &mut app::State) -> Option<Box<Mode>> {
        match e {
            Event::Key(k,false) => {
                match k {
                    KeyCode::Enter => {
                        let loc = app.ed.curr_loc(); 
                        app.ed.buf.borrow_mut().break_line(loc);
                        app.ed.invalidate_line(loc.1);
                        app.ed.insert_line(loc.1);
                        app.ed.cursor_col = 0;
                        app.ed.move_cursor((0, 1));
                        None
                    }
                    KeyCode::Delete => {
                        let loc = app.ed.curr_loc();
                        app.ed.buf.borrow_mut().delete_char(loc);
                        app.ed.invalidate_line(loc.1);
                        None
                    }
                    KeyCode::Backspace => {
                        let loc = app.ed.curr_loc();
                        app.ed.buf.borrow_mut().delete_char(loc);
                        app.ed.move_cursor((-1, 0));
                        app.ed.invalidate_line(loc.1);
                        None
                    }
                    KeyCode::Character(c) => {
                        if c.is_control() { None } else {
                        let loc = app.ed.curr_loc();
                        app.ed.buf.borrow_mut().insert_char(loc, c);
                        app.ed.move_cursor((1, 0));
                        app.ed.invalidate_line(loc.1);
                        None
                        }
                    }
                    KeyCode::Up => { app.ed.make_movement(Movement::Line(true)); None }
                    KeyCode::Down => { app.ed.make_movement(Movement::Line(false)); None }
                    KeyCode::Left => { app.ed.make_movement(Movement::Char(false)); None }
                    KeyCode::Right => { app.ed.make_movement(Movement::Char(true)); None }
                    KeyCode::Escape => { Some(Box::new(NormalMode::new())) }
                    _ => None
                }
            }
            _ => None
        }
    }

    fn status_tag(&self) -> &str { "INSERT" }
}
