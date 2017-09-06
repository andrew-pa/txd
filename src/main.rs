#![feature(box_patterns)]
#![feature(str_escape)]
extern crate runic;
// txd: a text editor🖳

mod buffer;
mod mode;
mod res;
mod app;
mod movement;

use runic::{App, Window as SystemWindow};
use app::*;

fn main() {
    SystemWindow::new("txd", 1280, 640, TxdApp::init).expect("create window!").show();
}
