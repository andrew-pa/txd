use runic::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;
use std::env;
use std::collections::HashMap;

use buffer::Buffer;
use res::Resources;
use lsp::LanguageServer;
use mode;

use winit::Event;
use regex::Regex;

use super::ConfigError;


#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct ClipstackId(pub char);

pub struct State {
    pub bufs: Vec<Rc<RefCell<Buffer>>>,
    pub res: Rc<RefCell<Resources>>,
    pub last_buffer: usize,
    pub current_buffer: usize,
    pub clipstacks: HashMap<ClipstackId, Vec<String>>,
    pub should_quit: bool,
    pub language_servers: Vec<(Regex, Rc<RefCell<LanguageServer>>)>,
    pub status_text: Option<String>
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
    
    pub fn language_server_for_file_type(&mut self, file_ext: &str) -> Result<Option<Rc<RefCell<LanguageServer>>>, Box<Error>> {
        for (ref test, ref lsp) in self.language_servers.iter() {
            if test.is_match(file_ext) {
                return Ok(Some(lsp.clone()));
            }
        }
        if let Some(cfgs) = self.res.borrow().config.as_ref().and_then(|c| c.get("language-server")).and_then(|c| c.as_array()) {
            for cfg in cfgs {
                let test = Regex::new(cfg.get("file-extention").ok_or(ConfigError::Missing("language server file extention regex"))?.as_str().ok_or(ConfigError::Invalid("language server file extention regex"))?)?;
                if test.is_match(file_ext) {
                    let lsp = Rc::new(RefCell::new(LanguageServer::new(&cfg)?));
                    self.language_servers.push((test, lsp.clone()));
                    return Ok(Some(lsp.clone()));
                }
            }
        }
        Ok(None)
    }
}

use std::path::{Path, PathBuf};

pub struct TxdApp {
    state: State,
    last_err: Option<Box<Error>>,
    mode: Box<mode::Mode>,
}

impl TxdApp {
    pub fn init(mut rx: &mut RenderContext) -> TxdApp {
        use std::fs::File;
        use std::io::Read;
        use toml::Value;
        let (config, le) : (Option<Value>, Option<Box<Error>>) = match File::open("config.toml") {
            Ok(mut f) => {
                let mut config_text = String::new();
                if let Err(e) = f.read_to_string(&mut config_text) {
                    (None, Some(Box::new(e)))
                } else {
                    match config_text.parse::<Value>() {
                        Ok(v) => (Some(v), None),
                        Err(e) => (None, Some(Box::new(e)))
                    } 
                }
            }
            Err(e) => (None, Some(Box::new(e)))
        };
        if let Some(ref e) = le {
            println!("config error {:?}", e);
        }

        let res = Rc::new(RefCell::new(Resources::new(rx, config).expect("create resources")));
        let buf = Rc::new(RefCell::new(Buffer::new(res.clone())));
                //env::args().nth(1).map_or_else(|| Buffer::new(res.clone()),
                //|p| Buffer::load(Path::new(&p), res.clone()).expect("open file"))  ));
        let cmd = Rc::new(RefCell::new(Buffer::new(res.clone())));
        { cmd.borrow_mut().show_cursor = false; }
        //println!("cd = {}, canoncd = {}", ::std::env::current_dir().unwrap().display(),
        //    ::std::env::current_dir().unwrap().canonicalize().unwrap().display());
        TxdApp {
            state: State {
                bufs: vec![cmd, buf],
                current_buffer: 1, last_buffer: 1,
                clipstacks: HashMap::new(), res,
                should_quit: false,
                language_servers: Vec::new(),
                status_text: None
            },
            mode: Box::new(mode::NormalMode::new()), last_err: le,
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

    fn paint(&mut self, rx: &mut RenderContext) {
        for lsp in self.state.language_servers.iter() {
            let st = &mut self.state.status_text;
            lsp.1.borrow_mut().process_notifications(|n| {
                match n["method"].as_str() {
                    Some("window/progress") => {
                        if n["params"].has_key("done") {
                            *st = None;
                        } else {
                            *st = Some(format!("{}: {}", n["params"]["title"], n["params"]["message"]));
                        }
                    },
                    Some(_) => println!("unknown notification {:?}", n),
                    None => println!("invalid notification {:?}", n)
                }
            });
        }

        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        let bnd = rx.bounds();
        let res = self.state.res.borrow();

        let mode_tag_tl = rx.new_text_layout(self.mode.status_tag(), &res.font, bnd.w, bnd.h).expect("create mode text layout");
        let mtb = mode_tag_tl.bounds();

        //draw buffer line
        rx.set_color(Color::rgb(0.25, 0.22, 0.2));
        rx.fill_rect(Rect::xywh(0.0, 0.0, bnd.w, mtb.h));
        rx.set_color(Color::rgb(0.1, 0.44, 0.5));
        rx.draw_text(Rect::xywh(4.0, 0.0, bnd.w, mtb.h), "txd", &res.font);
        {
        let mut x = 48.0;
        for (i, b) in self.state.bufs.iter().enumerate() {
            let tl = rx.new_text_layout(&format!("[{} {}]", i, 
                     b.borrow().fs_loc.as_ref().map_or_else(|| String::from("*"),
                        |p| format!("{}", p.strip_prefix(::std::env::current_dir().unwrap().as_path()).unwrap_or(p).display()) ),
), &res.font, bnd.w, bnd.h).expect("create text layout");
            if i == self.state.current_buffer {
                rx.set_color(Color::rgb(0.80, 0.44, 0.1));
            } else {
                rx.set_color(Color::rgb(0.50, 0.44, 0.1));
            }
            rx.draw_text_layout(Point::xy(x, 0.0), &tl);
            x += tl.bounds().w;
        }
        }

        let buf_ = self.state.buf();
        let mut buf = buf_.borrow_mut();
        buf.paint(rx, Rect::xywh(4.0, 4.0 + mtb.h*1.1, bnd.w-4.0, bnd.h-mtb.h*3.2));

        //draw status line
        let status_y = bnd.h-mtb.h*2.2;
        rx.set_color(Color::rgb(0.25, 0.22, 0.2));
        rx.fill_rect(Rect::xywh(0.0, status_y-0.5, bnd.w, mtb.h));
        rx.set_color(Color::rgb(0.4, 0.6, 0.0));
        /*rx.draw_text(Rect::xywh(4.0, bnd.h-35.0, bnd.w, 18.0), self.mode.status_tag(), &res.font);*/
        rx.draw_text_layout(Point::xy(4.0, status_y), &mode_tag_tl);
        rx.set_color(Color::rgb(0.9, 0.4, 0.0));
        rx.draw_text(Rect::xywh(100.0, status_y, bnd.w, 18.0),
                     &buf.fs_loc.as_ref().map_or_else(|| String::from("[new file]"),
                        |p| format!("{}", p.strip_prefix(::std::env::current_dir().unwrap().as_path()).unwrap_or(p).display()) ),
                     &res.font);
        if let Some(ref s) = self.state.status_text {
            rx.draw_text(Rect::xywh(600.0, status_y, bnd.w, 18.0), &s, &res.font);
        }
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
            rx.draw_text(Rect::xywh(bnd.w-200.0, status_y + mtb.h, bnd.w, 28.0), cmd,
                        &res.font);
        }
        self.state.bufs[0].borrow_mut().paint(rx, Rect::xywh(4.0, status_y + mtb.h, bnd.w-200.0, 50.0));
    }
}


