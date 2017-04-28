use pancurses::*;
use mode::*;
use std::path::{Path,PathBuf};

pub struct CommandMode {
    buf: String, old_cur: (usize,usize)
}

impl CommandMode {
    pub fn new(x: usize, y: usize) -> CommandMode {
        CommandMode {
            buf: String::from(""),
            old_cur: (x,y)
        }
    }

    fn run_command(&mut self, s: &mut State) -> Option<Box<Mode>> {
        self.buf.trim();
        if self.buf.chars().next() == Some('q') { s.should_quit = true; None }
        else if self.buf.chars().next() == Some('e') {
            s.cur_buf = s.buffers.len();
            s.buffers.push(Buffer::load(Path::new(&self.buf[1..])));
            Some(Box::new(NormalMode{}))
        } else if self.buf.chars().next() == Some('b') {
            self.buf.remove(0);
            s.cur_buf = self.buf.parse::<usize>().unwrap();
            Some(Box::new(NormalMode{}))
        } else if self.buf.chars().next() == Some('w') {
            self.buf.remove(0);
            if self.buf.len() > 0 { s.buffers[s.cur_buf].fs_loc = Some(PathBuf::from(&self.buf)); }
            s.buffers[s.cur_buf].sync_disk();
            Some(Box::new(NormalMode{}))
        } else { None }
    }
}

impl Mode for CommandMode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>> {
        match i {
            Input::Character('\x1B') => {
                s.cur_x = self.old_cur.0;
                s.cur_y = self.old_cur.1;
                Some(Box::new(NormalMode{}))
            },
            Input::Character('\n')=> {
                s.cur_x = self.old_cur.0;
                s.cur_y = self.old_cur.1;
                self.run_command(s).or(Some(Box::new(NormalMode{})))
            }
            Input::KeyBackspace => {
                    self.buf.remove(s.cur_x - 1);
                    s.cur_x -= 1;
                    None
            },
            Input::Character(c) => {
                if !c.is_control() {
                    self.buf.insert(s.cur_x, c);
                    s.cur_x += 1
                } else if c == '\x08' {
                    self.buf.remove(s.cur_x - 1);
                    s.cur_x -= 1
                }
                None
            },
            _ => None
        }
    }

    fn status_text(&self) -> &str { "COMMAND" }

    fn draw(&self, win: &Window) {
        win.mvprintw(win.get_max_y()-1, 0, &self.buf);
    }
}



