use pancurses::*;
use buffer::*;
use mode::*;

pub struct NormalMode {}

impl Mode for NormalMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('h') => { s.cur_buf().move_loca(-1,  0); None },
            Input::Character('j') => { s.cur_buf().move_loca( 0,  1); None },
            Input::Character('k') => { s.cur_buf().move_loca( 0, -1); None },
            Input::Character('l') => { s.cur_buf().move_loca( 1,  0); None },
            Input::Character('i') => Some(Box::new(insert::InsertMode{})),
            Input::Character('o') => { 
                s.cur_buf().insert_line(None);
                Some(Box::new(insert::InsertMode{}))
            },
            Input::Character(':') => {
                let r : Box<Mode> = Box::new(command::CommandMode::new(s.cur_x, s.cur_y));
                s.cur_x = 0; s.cur_y = s.win.get_max_y()as usize-1;
                Some(r)
            },
            _ => None
        }
    }
    fn status_text(&self) -> &str { "NORMAL" }
    fn draw(&self, win: &Window) {
    }
}
