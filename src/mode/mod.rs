
use runic;
use app;

pub trait Mode {
    fn event(&mut self, e: runic::Event, app: &mut app::State, win: runic::WindowRef) -> Option<Box<Mode>>;
    fn status_tag(&self) -> &str;
    fn pending_command(&self) -> Option<&str> { None }
}

mod normal;
mod insert;
mod command;
pub use self::normal::NormalMode;
pub use self::insert::InsertMode;
pub use self::command::CommandMode;
