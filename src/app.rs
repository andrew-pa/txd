use runic::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;
use std::env;
use std::collections::HashMap;

use buffer::Buffer;
use res::Resources;
use mode;

use winit::Event;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct ClipstackId(pub char);

pub struct State {
    pub bufs: Vec<Rc<RefCell<Buffer>>>,
    pub res: Rc<RefCell<Resources>>,
    pub last_buffer: usize,
    pub current_buffer: usize,
    pub clipstacks: HashMap<ClipstackId, Vec<String>>,
    pub should_quit: bool
}

impl State {
    pub fn buf(&self) -> Rc<RefCell<Buffer>> {
        self.bufs[self.current_buffer].clone()
    }

    pub fn mutate_buf<R, F: FnOnce(&mut Buffer)->R>(&mut self, f: F) -> R {
        f(&mut self.bufs[self.current_buffer].borrow_mut())
    }

    pub fn push_clip(&mut self, id: &ClipstackId, s: String) {
        let mut stack = self.clipstacks.entry(id.clone()).or_insert(Vec::new());
        stack.push(s);
    }

    pub fn top_clip(&self, id: &ClipstackId) -> Option<String> {
        self.clipstacks.get(id).and_then(|sk| sk.last()).map(Clone::clone)
    }

    pub fn pop_clip(&mut self, id: &ClipstackId) -> Option<String> {
        self.clipstacks.get_mut(id).and_then(|sk| sk.pop())
    }

    pub fn move_to_buffer(&mut self, ix: usize) {
        self.last_buffer = self.current_buffer;
        self.current_buffer = ix;
    }
}

use std::path::{Path, PathBuf};

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
        println!("cd = {}, canoncd = {}", ::std::env::current_dir().unwrap().display(), ::std::env::current_dir().unwrap().canonicalize().unwrap().display());
        TxdApp { state: State {
                bufs: vec![cmd, buf],
                current_buffer: 1, last_buffer: 1,
                clipstacks: HashMap::new(), res,
                should_quit: false
            }, mode: Box::new(mode::NormalMode::new()), last_err: None
        }
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event) -> bool {
        match e {
            Event::WindowEvent { event: we, .. } => {
                let nxm = self.mode.event(we, &mut self.state);
                match nxm {
                    Ok(Some(new_mode)) => { if self.last_err.is_some() { self.last_err = None; } self.mode = new_mode }
                    Ok(None) => {}
                    Err(err) => { println!("error: {}", err); self.last_err = Some(err); self.mode = Box::new(mode::NormalMode::new()); }
                } 
            },
            _ => { }
        }
        self.state.should_quit
    }

    fn paint(&mut self, mut rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        let bnd = rx.bounds();
        let mut buf_ = self.state.buf();
        let mut buf = buf_.borrow_mut();
        let res = self.state.res.borrow();
        buf.paint(rx, Rect::xywh(4.0, 4.0, bnd.w-4.0, bnd.h-34.0));
        
        //draw status line
        let mode_tag_tl = rx.new_text_layout(self.mode.status_tag(), &res.font, bnd.w, bnd.h).expect("create mode text layout");
        let mtb = mode_tag_tl.bounds();
        let status_y = bnd.h-mtb.h*2.2;

        rx.set_color(Color::rgb(0.25, 0.22, 0.2));
        rx.fill_rect(Rect::xywh(0.0, status_y-0.5, bnd.w, mtb.h));
        rx.set_color(Color::rgb(0.4, 0.6, 0.0));
        /*rx.draw_text(Rect::xywh(4.0, bnd.h-35.0, bnd.w, 18.0), self.mode.status_tag(), &res.font);*/
        rx.draw_text_layout(Point::xy(4.0, status_y), &mode_tag_tl);
        rx.set_color(Color::rgb(0.9, 0.4, 0.0));
        rx.draw_text(Rect::xywh(100.0, status_y, bnd.w, 18.0),
                     &buf.fs_loc.as_ref().map_or(String::from(""), |p| format!("{}", p.display())),
                     &res.font);
        rx.set_color(Color::rgb(0.0, 0.6, 0.4));
        rx.draw_text(Rect::xywh(bnd.w-200.0, status_y, bnd.w, 18.0),
                     &format!("ln {} col {}", buf.cursor_line, buf.cursor_col),
                     &res.font);
        if let Some(ref err) = self.last_err {
            rx.set_color(Color::rgb(0.9, 0.2, 0.0));
            rx.draw_text(Rect::xywh(4.0, status_y + mtb.h, bnd.w, 18.0),
                &format!("error: {}", err),
                &res.font);
        }
        //draw command line
        if let Some(cmd) = self.mode.pending_command() {
            rx.set_color(Color::rgb(0.8, 0.8, 0.8));
            rx.draw_text(Rect::xywh(bnd.w-200.0, status_y + mtb.h, bnd.w, 18.0), cmd,
                        &res.font);
        }
        self.state.bufs[0].borrow_mut().paint(rx, Rect::xywh(4.0, status_y + mtb.h, bnd.w-200.0, 20.0));
    }
}


