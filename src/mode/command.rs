use vtctl::*;
use mode::*;
use std::path::{Path,PathBuf};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct CommandError {
    msg: String
}

impl Error for CommandError {
    fn description(&self) -> &str { "invalid command" }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {}", self.msg)
    }
}

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

    fn run_command(&mut self, s: &mut State) -> Result<Option<Box<Mode>>, Box<Error>> {
        self.buf.trim();
        if self.buf.chars().next() == Some('q') { s.should_quit = true; Ok(None) }
        else if self.buf.chars().next() == Some('e') {
            s.cur_buf = s.buffers.len();
            s.buffers.push(Buffer::load(Path::new(&self.buf[1..]))?);
            Ok(Some(Box::new(NormalMode{})))
        } else if self.buf.chars().next() == Some('b') {
            self.buf.remove(0);
            let cbf = self.buf.parse::<usize>()?;
            if cbf > s.buffers.len() { return Err(Box::new(CommandError { msg: String::from("index beyond open buffers") })); }
            s.cur_buf = cbf;
            Ok(Some(Box::new(NormalMode{})))
        } else if self.buf.chars().next() == Some('w') {
            self.buf.remove(0);
            if self.buf.len() > 0 { s.buffers[s.cur_buf].fs_loc = Some(PathBuf::from(&self.buf)); }
            s.buffers[s.cur_buf].sync_disk()?;
            Ok(Some(Box::new(NormalMode{})))
        } else { Ok(None) }
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
                match self.run_command(s) {
                    Ok(x) => x.or(Some(Box::new(NormalMode{}))),
                    Err(e) => {
                        s.usr_err = Some(e);
                        Some(Box::new(NormalMode{}))
                    }
                }
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
        win.write_at(win.size().1-1, 0, &self.buf);
    }
}



