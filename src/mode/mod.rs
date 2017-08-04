
use runic;
use bufferview;

pub trait Mode {
    fn event(&mut self, e: runic::Event, bv: &mut bufferview::BufferView) -> Option<Box<Mode>>;
    fn status_tag(&self) -> &str;
}

mod normal;
pub use self::normal::*;
