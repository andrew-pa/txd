
use runic;
use bufferview;

pub trait Mode {
    fn event(&mut self, e: runic::Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>>;
    fn status_tag(&self) -> &str;
}

mod normal;
mod insert;
mod command;
pub use self::normal::NormalMode;
pub use self::insert::InsertMode;
pub use self::command::CommandMode;
