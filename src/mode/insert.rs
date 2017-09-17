
use super::*;
use winit::*;
use movement::Movement;

pub struct InsertMode {
    target_buffer: Option<usize>
}

impl InsertMode {
    pub fn new() -> InsertMode { InsertMode { target_buffer: None } }
    pub fn new_with_target(target: usize) -> InsertMode { InsertMode { target_buffer: Some(target) } }
}

impl Mode for InsertMode {
    fn event(&mut self, e: WindowEvent, app: &mut app::State) -> Result<Option<Box<Mode>>, Box<Error>> {
        let mut buf_ = match self.target_buffer {
            Some(target) => app.bufs[target].clone(),
            None => app.buf(),
        };
        let mut buf = buf_.borrow_mut();
        let cloc = buf.curr_loc();

        match e {
            WindowEvent::ReceivedCharacter(c) => {
                if c.is_control() { Ok(None) } else {
                    buf.insert_char(c);
                    Ok(None)
                }
            },
            WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(k), .. }, .. } => {
                match k {
                    VirtualKeyCode::Return => {
                        buf.break_line();
                        Ok(None)
                    }
                    VirtualKeyCode::Delete => {
                        buf.delete_char();
                        Ok(None)
                    }
                    VirtualKeyCode::Back => {
                        if cloc.0 != 0 {
                            buf.delete_char();
                            buf.move_cursor((-1, 0));
                        }
                        Ok(None)
                    }
                    VirtualKeyCode::Tab => {
                        buf.insert_tab();
                        Ok(None)
                    }
                    VirtualKeyCode::Up => { buf.make_movement(Movement::Line(true)); Ok(None) }
                    VirtualKeyCode::Down => { buf.make_movement(Movement::Line(false)); Ok(None) }
                    VirtualKeyCode::Left => { buf.make_movement(Movement::Char(false)); Ok(None) }
                    VirtualKeyCode::Right => { buf.make_movement(Movement::Char(true)); Ok(None) }
                    VirtualKeyCode::Escape => { Ok(Some(Box::new(NormalMode::new()))) }
                    _ => Ok(None)
                }
            }
            _ => Ok(None)
        }
    }

    fn status_tag(&self) -> &str { "INSERT" }
}
