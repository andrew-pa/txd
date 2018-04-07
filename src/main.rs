#![feature(box_patterns)]
#![feature(str_escape)]
#![feature(slice_patterns)]
extern crate runic;
extern crate winit;
extern crate futures;
extern crate toml;
extern crate json;
extern crate mio;
extern crate mio_named_pipes;

// txd: a text editor🖳

mod buffer;
mod mode;
mod res;
mod app;
mod movement;
mod lsp;
//mod fs_util;

use runic::*;
use winit::*;
use app::*;

use std::error::Error;

use futures::Future;

#[derive(Debug)]
enum ConfigError {
    Parse(Box<Error>),
    Missing(&'static str),
    Invalid(&'static str)
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match self {
            &ConfigError::Parse(_) => "parse error",
            &ConfigError::Missing(_) => "incomplete config",
            &ConfigError::Invalid(_) => "invalid config"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &ConfigError::Parse(ref e) => Some(e.as_ref()),
            _ => None
        }
    }
}

use std::fmt::*;
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &ConfigError::Parse(ref e) => write!(f, "parse error: {}", e),
            &ConfigError::Missing(v) => write!(f, "missing config value \"{}\"", v),
            &ConfigError::Invalid(v) => write!(f, "invalid config value \"{}\"", v)
        }
    }
}

fn main() {
    runic::init();
    let mut evl = EventsLoop::new();
    let mut window = WindowBuilder::new().with_dimensions(1280, 640).with_title("txd").build(&evl).expect("create window!");
    let mut rx = RenderContext::new(&mut window).expect("create render context!");
    let mut app = TxdApp::init(&mut rx);
    app.run(&mut rx, &mut evl);
}
