#![feature(box_patterns)]
#![feature(str_escape)]
extern crate runic;
extern crate winit;

// txd: a text editor🖳

mod buffer;
mod mode;
mod res;
mod app;
mod movement;

use runic::*;
use winit::*;
use app::*;

fn main() {
    runic::init();
    let mut evl = EventsLoop::new();
    let mut window = WindowBuilder::new().with_dimensions(1280, 640).with_title("txd").build(&evl).expect("create window!");
    let mut rx = RenderContext::new(&mut window).expect("create render context!");
    let mut app = TxdApp::init(&mut rx);
    app.run(&mut rx, &mut evl);
}
