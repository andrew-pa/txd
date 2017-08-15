
use super::*;
use runic::{Event, KeyCode};

pub struct CommandMode {
    buf: String
}

impl CommandMode {
    pub fn new() -> CommandMode {
        CommandMode { buf: String::new() }
    }
}

impl Mode for CommandMode {
    fn event(&mut self, e: Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>> {
        match e {
            Event::Key(k, false) => match k {
                KeyCode::Character(c) => { self.buf.push(c); None }
                KeyCode::Enter => {
                    self.buf.clear();
                    None
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
