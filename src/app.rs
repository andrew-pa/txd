use runic::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;
use std::path::Path;
use std::env;
use std::collections::HashMap;

use buffer::Buffer;
use res::Resources;
use mode;

#[derive(Debug,PartialEq,Eq,Hash)]
pub struct RegisterId(pub char);

pub struct State {
    pub bufs: Vec<Rc<RefCell<Buffer>>>,
    pub res: Rc<RefCell<Resources>>,
    pub current_buffer: usize,
    pub registers: HashMap<RegisterId, String>
}

impl State {
    pub fn buf(&self) -> Rc<RefCell<Buffer>> {
        self.bufs[self.current_buffer].clone()
    }

    pub fn mutate_buf<R, F: FnOnce(&mut Buffer)->R>(&mut self, f: F) -> R {
        f(&mut self.bufs[self.current_buffer].borrow_mut())
    }
}

pub struct TxdApp {
    state: State,
    last_err: Option<Box<Error>>,
    mode: Box<mode::Mode>
}

impl TxdApp {
    pub fn init(mut rx: &mut RenderContext) -> TxdApp {
        let res = Rc::new(RefCell::new(Resources::new(rx).expect("create resources")));
        let buf = Rc::new(RefCell::new(
                env::args().nth(1).map_or_else(|| Buffer::new(res.clone()),
                                                |p| Buffer::load(Path::new(&p), res.clone()).expect("open file"))  ));
        let cmd = Rc::new(RefCell::new(Buffer::new(res.clone())));
        { cmd.borrow_mut().show_cursor = false; }
        TxdApp { state: State { bufs: vec![cmd, buf], current_buffer: 1, registers: HashMap::new(), res }, mode: Box::new(mode::NormalMode::new()), last_err: None }
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event, win: WindowRef) {
        let nxm = self.mode.event(e, &mut self.state, win);
        match nxm {
            Ok(Some(new_mode)) => { if self.last_err.is_some() { self.last_err = None; } self.mode = new_mode }
            Ok(None) => {}
            Err(err) => { println!("error: {}", err); self.last_err = Some(err); self.mode = Box::new(mode::NormalMode::new()); }
        }
    }

    fn paint(&mut self, mut rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        let bnd = rx.bounds();
        let mut buf_ = self.state.buf();
        let mut buf = buf_.borrow_mut();
        let res = self.state.res.borrow();
        buf.paint(rx, Rect::xywh(4.0, 4.0, bnd.w-4.0, bnd.h-34.0));
        
        //draw status line
        rx.fill_rect(Rect::xywh(0.0, bnd.h-34.0, bnd.w, 18.0), Color::rgb(0.25, 0.22, 0.2));
        rx.draw_text(Rect::xywh(4.0, bnd.h-35.0, bnd.w, 18.0), self.mode.status_tag(),
                     Color::rgb(0.4, 0.6, 0.0), &res.font);
        rx.draw_text(Rect::xywh(100.0, bnd.h-35.0, bnd.w, 18.0), &buf.fs_loc.as_ref().map_or(String::from(""), |p| format!("{}", p.display())),
                     Color::rgb(0.9, 0.4, 0.0), &res.font);
        rx.draw_text(Rect::xywh(bnd.w-200.0, bnd.h-35.0, bnd.w, 18.0),
                     &format!("ln {} col {}", buf.cursor_line, buf.cursor_col),
                     Color::rgb(0.0, 0.6, 0.4), &res.font);
        if let Some(ref err) = self.last_err {
            rx.draw_text(Rect::xywh(4.0, bnd.h-18.0, bnd.w, 18.0),
                &format!("error: {}", err),
                Color::rgb(0.9, 0.2, 0.0), &res.font);
        }
        //draw command line
        if let Some(cmd) = self.mode.pending_command() {
            rx.draw_text(Rect::xywh(bnd.w-200.0, bnd.h-18.0, bnd.w, 18.0), cmd,
            Color::rgb(0.8, 0.8, 0.8), &res.font);
        }
        self.state.bufs[0].borrow_mut().paint(rx, Rect::xywh(4.0, bnd.h-18.0, bnd.w-200.0, 20.0));
    }
}


