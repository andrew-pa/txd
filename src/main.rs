#![feature(vec_resize_default)]
extern crate runic;

use runic::{App, Window as SystemWindow, Event, RenderContext, Color, Point, Rect, Font, TextLayout, KeyCode};
use std::error::Error;

/*
    System-Window [
        Window [ TextBuffer ]
        Window [ Image ]
        Window [ Terminal ]
    ]

    Window [ Client-Area; Status-Bar ]
*/

struct Resources {
    main_font: Font
}

impl Resources {
    fn init(rx: &mut RenderContext) -> Result<Resources, Box<Error>> {
        Ok(Resources {
            main_font: Font::new(rx, String::from("Consolas"), 14.0, runic::FontWeight::Regular, runic::FontStyle::Normal)?
        })
    }
}

trait Interface {
    fn event(&mut self, e: Event) -> bool; //handled?
    fn paint(&mut self, rx: &mut RenderContext, res: &Resources);
}

struct Window {
    area: Rect,
    content: Box<Interface>
}

impl Window {
}

impl Interface for Window {
    fn event(&mut self, e: Event) -> bool {
        match e {
            Event::Resize(_,_,s) => { self.area.w = s.x; self.area.h = s.y; },
            _ => {}
        }
        self.content.event(e)
    }

    fn paint(&mut self, rx: &mut RenderContext, res: &Resources) {
        rx.stroke_rect(self.area, Color::rgb(1.0, 1.0, 1.0), 1.0);
        rx.translate(Point::xy(self.area.x, self.area.y));
        self.content.paint(rx, res);
        rx.translate(Point::xy(0.0, 0.0));
    }
}

struct Buffer {
    lines: Vec<String>
}

struct TextEdit {
    buf: Buffer,
    line_layouts: Vec<Option<runic::TextLayout>>,
    viewport: usize
}

impl TextEdit {
    fn new(buf: Buffer) -> TextEdit {
        let mut s = TextEdit {
            buf: buf, line_layouts: Vec::new(), viewport: 0
        };
        s.line_layouts.resize_default(s.buf.lines.len());
        s
    }
}

impl Interface for TextEdit {
    fn event(&mut self, e: Event) -> bool { false }
    fn paint(&mut self, rx: &mut RenderContext, res: &Resources) {
        let mut dp = Point::xy(4.0,4.0);
        for line in self.viewport..(self.buf.lines.len()) {
            if let None = self.line_layouts[line] {
                self.line_layouts[line] = Some(runic::TextLayout::new(rx, &self.buf.lines[line], &res.main_font, 512.0, 32.0).expect("create layout"));
            }
            let layout = self.line_layouts[line].as_ref().unwrap();
            rx.draw_text_layout(dp, layout, Color::rgb(0.9, 0.9, 0.9));
            let line_size = layout.bounds();
            dp.y += line_size.h;
        }
    }
}

struct TestInterface;

impl Interface for TestInterface {
    fn event(&mut self, e: Event) -> bool { false }
    fn paint(&mut self, rx: &mut RenderContext, res: &Resources) {
        rx.draw_text(Rect::xywh(4.0,4.0,64.0,32.0), "Hello, World!",  Color::rgb(0.9, 0.5, 0.0), &res.main_font);
    }
}

#[derive(Clone, Debug)]
enum WindowTree {
    Split {
       first: Box<WindowTree>, second: Box<WindowTree>,
       percent: f32, orientation: u8
    },
    Leaf(usize) //index into window list
}

impl WindowTree {
    fn find_split_containing_window(&mut self, window_index: usize) -> Option<(&mut WindowTree, u8)> {
        match self {
            &mut WindowTree::Split { ref mut first, ref mut second, .. } => {
                match **first {
                    WindowTree::Leaf(idx) => { if idx == window_index { Some((self, 0)) } else { None } },
                    WindowTree::Split{..} => { first.find_split_containing_window(window_index) }
                }.or_else(move || match **second {
                    WindowTree::Leaf(idx) => { if idx == window_index { Some((self, 1)) } else { None } },
                    WindowTree::Split{..} => { second.find_split_containing_window(window_index) }
                })
            },
            &mut WindowTree::Leaf(_) => None
        }
    }
}

struct TxdApp {
    windows: Vec<Window>,
    active_window: usize,
    layout: WindowTree,
    res: Resources,
    ctrl_down: bool, ctrl_key: Option<char>
}

impl TxdApp {
    fn init(rx: &mut RenderContext) -> TxdApp {
        let mut app = TxdApp { 
            windows: vec![ 
                Window{area: rx.bounds(), content: Box::new(TestInterface)},
                Window{area: rx.bounds(), content: Box::new(TestInterface)},
                Window{area: rx.bounds(), content: Box::new(TextEdit::new(Buffer { lines: vec![
                    String::from("Hello, World!"),
                    String::from("This is a text editor!"),
                    String::from("A third line!")
                ]}))}
            ], 
            active_window: 0, ctrl_down: false, ctrl_key: None,
            res: Resources::init(rx).expect("load resources"),
            layout: WindowTree::Split { 
                first: Box::new(WindowTree::Split { 
                    first: Box::new(WindowTree::Leaf(0)),
                    second: Box::new(WindowTree::Leaf(1)),
                    percent: 0.7, orientation: 1
                }), 
                second: Box::new(WindowTree::Leaf(2)), percent: 0.5, orientation: 0 }
        };
        let ly = app.layout.clone();
        app.recalculate_layout(rx.bounds(), &ly);
        app
    }

    fn recalculate_layout(&mut self, area: Rect, tree: &WindowTree) {
        match tree {
            &WindowTree::Leaf(idx) => {
                self.windows[idx].area.x = area.x;
                self.windows[idx].area.y = area.y;
                self.windows[idx].event(Event::Resize(0,0,Point::xy(area.w, area.h)));
            },
            &WindowTree::Split { ref first, ref second, percent, orientation } => {
                let (farea, sarea) = match orientation {
                    0 => (Rect::xywh(area.x, area.y, area.w*percent, area.h),
                          Rect::xywh(area.x+area.w*percent, area.y, area.w*(1.0-percent), area.h)),
                    1 => (Rect::xywh(area.x, area.y, area.w, area.h*percent),
                          Rect::xywh(area.x, area.y+area.h*percent, area.w, area.h*(1.0-percent))),
                    _ => panic!()
                };
                self.recalculate_layout(farea, &first);
                self.recalculate_layout(sarea, &second);
            }
        }
    }

    fn add_new_window(&mut self, orientation: u8) {
        println!("before = {:?}", self.layout);
        match self.layout.find_split_containing_window(self.active_window) {
            Some((&mut WindowTree::Split { ref mut first, ref mut second, .. }, side)) => {
                let new_window = self.windows.len();
                self.windows.push(Window { area: Rect::xywh(0.0,0.0,0.0,0.0), content: Box::new(TestInterface)});
                let new_split = Box::new(WindowTree::Split {
                    first: Box::new(WindowTree::Leaf(self.active_window)), second: Box::new(WindowTree::Leaf(new_window)),
                    orientation: orientation, percent: 0.5
                });
                if side == 0 { *first = new_split; } else { *second = new_split; }
            }
            _ => panic!("!")
        }
        println!("after = {:?}", self.layout);
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event) {
        let fw = self.active_window;
        if let Event::Resize(_,_,size) = e {
            let ly = self.layout.clone();
            self.recalculate_layout(Rect::xywh(0.0,0.0,size.x,size.y), &ly);
            return;
        }
        if self.windows[fw].event(e) { return; }
        /* handle events app-wide */
        match e {
            Event::Key(KeyCode::Ctrl, kd) => { if !kd { self.ctrl_key = None; } self.ctrl_down = kd; },
            Event::Key(KeyCode::RawCharacter(c), kd) => {
                if self.ctrl_down {
                    if kd {
                        if let Some(lc) = self.ctrl_key {
                            println!("ctl {} {}", lc, c);
                            match lc {
                                'W' => match c {
                                    'S' => { self.add_new_window(0); } // split current window vertically
                                    'E' => { self.add_new_window(1); } // split current window horizontally
                                    'H' => {} // move left
                                    'J' => {} // move down
                                    'K' => {} // move up
                                    'L' => {} // move right
                                    _ => {}
                                },
                                _ => {}
                            }
                            self.ctrl_key = None;
                        } else {
                            println!("ctl {} 1", c);
                            self.ctrl_key = Some(c);
                        } 
                    } else { self.ctrl_key = None; }
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        for w in self.windows.iter_mut() {
            w.paint(rx, &self.res);
        }
        let aw = self.active_window;
        rx.stroke_rect(self.windows[aw].area, Color::rgb(0.8, 0.6, 0.0), 2.0);
    }
}

fn main() {
    SystemWindow::new("txd", 640, 480, TxdApp::init).expect("create window!").show();
}
