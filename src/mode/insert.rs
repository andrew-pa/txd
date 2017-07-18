use vtctl::*;
use buffer::*;
use mode::*;

pub struct InsertMode {}

impl Mode for InsertMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::KeyLeft => { s.cur_buf().move_loca(-1, 0); None },
            Input::KeyRight => { s.cur_buf().move_loca(1, 0); None },
            Input::KeyUp => { s.cur_buf().move_loca(0, -1); None },
            Input::KeyDown => { s.cur_buf().move_loca(0, 1); None },
            Input::Character('\x1B') => Some(Box::new(NormalMode{})),
            Input::KeyBackspace => { s.cur_buf().backspace(); None },
            Input::KeyEnter => { s.cur_buf().break_line(); None },
            Input::Character(c) => {
                if !c.is_control() {
                    s.cur_buf().insert_char(c);
                } else if c == '\x08' {
                    s.cur_buf().backspace();
                } else if c == '\n' {
                    s.cur_buf().break_line();
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
