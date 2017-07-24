use std::error::Error;
use vtctl::*;
use buffer::*;

pub struct State {
    pub cur_x: usize, pub cur_y: usize,
    pub cur_buf: usize,
    pub buffers: Vec<Buffer>, pub usr_err: Option<Box<Error>>,
    pub should_quit: bool
}

impl State {
    pub fn init() -> State {
        State {
            cur_x: 0,
            cur_y: 0,
            buffers: Vec::new(),
            cur_buf: 0,
            should_quit: false, usr_err: None
        }
    }

    pub fn current_buffer(&self) -> &Buffer {
        &self.buffers[self.cur_buf]
    }

    pub fn cur_buf(&mut self) -> &mut Buffer {
        &mut self.buffers[self.cur_buf]
    }
}

pub trait Mode {
    fn handle_input(&mut self, i: Input, s: &mut State) -> Option<Box<Mode>>;
    fn status_text(&self) -> &str;
    fn draw(&self, win: &mut Window) { }
}

pub mod normal;
pub mod insert;
pub mod command;
pub use self::normal::*;
pub use self::insert::*;
pub use self::command::*;
