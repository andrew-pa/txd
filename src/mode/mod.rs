
use runic;
use app;
use std::error::Error;

pub trait Mode {
    fn event(&mut self, e: runic::Event, app: &mut app::State, win: runic::WindowRef) -> Result<Option<Box<Mode>>, Box<Error>>;
    fn status_tag(&self) -> &str;
    fn pending_command(&self) -> Option<&str> { None }
}

mod normal;
mod insert;
mod command;
pub use self::normal::NormalMode;
pub use self::insert::InsertMode;
pub use self::command::CommandMode;
