
use super::*;
use runic::{Event, KeyCode};

pub struct CommandMode;

impl Mode for CommandMode {
    fn event(&mut self, e: Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>> {
        match e {
            Event::Key(k, false) => match k {
                KeyCode::Escape => Some(Box::new(NormalMode::new())),
                _ => None,
            }
            _ => None,
        }
    }
    fn status_tag(&self) -> &str { "COMMAND" }
}
