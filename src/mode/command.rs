
use super::*;
use runic::{Event, KeyCode, WindowRef};
use std::rc::Rc;
use std::cell::RefCell;
use buffer::Buffer;
use std::path::Path;

pub struct CommandMode {
    inserter: InsertMode
}

impl CommandMode {
    pub fn new(app: &mut app::State) -> CommandMode {
        app.bufs[0].borrow_mut().show_cursor = true;
        CommandMode { inserter: InsertMode::new_with_target(0) }
    }

    pub fn execute(&self, app: &mut app::State, win: WindowRef) -> Option<Box<Mode>> {
        let cmd = { 
            app.bufs[0].borrow().lines.last().unwrap().clone()
        };
        match cmd.chars().next() {
            Some('q') => { win.quit(); Some(Box::new(NormalMode::new())) },
            Some('w') => {
                app.mutate_buf(|b| b.sync_disk()).expect("sync buffer to disk");
                Some(Box::new(NormalMode::new()))
            },
            Some('e') => {
                let (e, path) = cmd.split_at(1);
                app.bufs.push(Rc::new(RefCell::new(Buffer::load(Path::new(path.trim()), app.res.clone()).expect("load buffer"))));
                app.current_buffer = app.bufs.len()-1;
                Some(Box::new(NormalMode::new()))
            },
            Some('b') => {
                let (b, num) = cmd.split_at(1);
                app.current_buffer = num.trim().parse::<usize>().expect("valid integer");
                Some(Box::new(NormalMode::new()))
            },
            _ => None
        }
    }
}

impl Mode for CommandMode {
    fn event(&mut self, e: Event, app: &mut app::State, win: WindowRef) -> Option<Box<Mode>> {
        match e {
            Event::Key(k, false) => match k {
                KeyCode::Enter => {
                    let r = self.execute(app, win);
                    let mut buf_ = &app.bufs[0];
                    let mut buf = buf_.borrow_mut();
                    let len = buf.lines.len();
                    buf.show_cursor = false;
                    buf.clear();
                    r
                }
                KeyCode::Escape => {
                    let mut buf_ = &app.bufs[0];
                    let mut buf = buf_.borrow_mut();
                    buf.show_cursor = false;
                    buf.clear();
                    Some(Box::new(NormalMode::new()))
                }
                _ => self.inserter.event(e, app, win),
            }
            _ => self.inserter.event(e, app, win),
        }
    }
    fn status_tag(&self) -> &str { "COMMAND" }
}
