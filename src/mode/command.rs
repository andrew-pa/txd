
use super::*;
use runic::{Event, KeyCode, WindowRef};

pub struct CommandMode {
    buf: String
}

impl CommandMode {
    pub fn new() -> CommandMode {
        CommandMode { buf: String::new() }
    }

    pub fn execute(&self, app: &mut app::State, win: WindowRef) -> Option<Box<Mode>> {
        match self.buf.chars().next() {
            Some('q') => { win.quit(); Some(Box::new(NormalMode::new())) },
            Some('w') => {
                app.buf.borrow_mut().sync_disk().expect("sync buffer to disk");
                Some(Box::new(NormalMode::new()))
            },
            _ => None
        }
    }
}

impl Mode for CommandMode {
    fn event(&mut self, e: Event, app: &mut app::State, win: WindowRef) -> Option<Box<Mode>> {
        match e {
            Event::Key(k, false) => match k {
                KeyCode::Character(c) => { self.buf.push(c); None }
                KeyCode::Enter => {
                    let r = self.execute(app, win);
                    self.buf.clear();
                    r
                }
                KeyCode::Escape => Some(Box::new(NormalMode::new())),
                _ => None,
            }
            _ => None,
        }
    }
    fn status_tag(&self) -> &str { "COMMAND" }
    fn pending_command(&self) -> Option<&str> { if self.buf.len() > 0 { Some(&self.buf) } else { None } }
}
