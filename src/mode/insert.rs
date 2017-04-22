use pancurses::*;
use buffer::*;
use mode::*;

pub struct InsertMode {}

impl Mode for InsertMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('\x1B') => Some(Box::new(NormalMode{})),
            Input::Character(c) => {
                if !c.is_control() {
                    s.cur_buf().insert_char(c);
                } else if c == '\x08' {
                    s.cur_buf().backspace();
                }
                None
            },
            _ => None
        }
    }

    fn status_text(&self) -> &str { "INSERT" }

    fn draw(&self, win: &Window) {
    }
}
