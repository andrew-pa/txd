#![feature(vec_resize_default)]
#![feature(box_patterns)]
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
    fn paint(&self, rx: &mut RenderContext, res: &Resources);
}

struct Window {
    area: Rect,
    active: bool,
    content: Box<Interface>
}

use std::fmt;
impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Window")
    }
}

impl Window {
    fn new(content: Box<Interface>) -> Window {
        Window {
            area: Rect::xywh(0.0,0.0,0.0,0.0),
            active: false,
            content: content
        }
    }
}

impl Interface for Window {
    fn event(&mut self, e: Event) -> bool {
        match e {
            Event::Resize(_,_,s) => { self.area.w = s.x; self.area.h = s.y; },
            Event::MouseMove(p,_) => {
                self.active = self.area.contains(p);
            }
            _ => {}
        }
        self.content.event(e)
    }

    fn paint(&self, rx: &mut RenderContext, res: &Resources) {
        if self.active {
            rx.stroke_rect(Rect::xywh(self.area.x+6.0, self.area.y+6.0, self.area.w-12.0, self.area.h-12.0), Color::rgb(1.0, 0.0, 0.0), 2.0);
        }
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
    fn paint(&self, rx: &mut RenderContext, res: &Resources) {
        /*let mut dp = Point::xy(4.0,4.0);
        for line in self.viewport..(self.buf.lines.len()) {
            if let None = self.line_layouts[line] {
                self.line_layouts[line] = Some(runic::TextLayout::new(rx, &self.buf.lines[line], &res.main_font, 512.0, 32.0).expect("create layout"));
            }
            let layout = self.line_layouts[line].as_ref().unwrap();
            rx.draw_text_layout(dp, layout, Color::rgb(0.9, 0.9, 0.9))
            let line_size = layout.bounds();
            dp.y += line_size.h;
        }*/
    }
}

struct TestInterface;

impl Interface for TestInterface {
    fn event(&mut self, e: Event) -> bool { false }
    fn paint(&self, rx: &mut RenderContext, res: &Resources) {
        rx.draw_text(Rect::xywh(4.0,4.0,64.0,32.0), "Hello, World!",  Color::rgb(0.9, 0.5, 0.0), &res.main_font);
    }
}

use std::rc::Rc;

#[derive(Debug)]
enum WindowLayout {
    Split {
       children: (Box<WindowLayout>, Box<WindowLayout>),
       percent: f32, orientation: bool, bounds: Rect
    },
    Leaf(Rc<Window>)
}

impl WindowLayout {
    fn bounds(&self) -> Rect {
        match *self {
            WindowLayout::Leaf(ref win) => win.area,
            WindowLayout::Split { bounds, .. } => bounds
        }
    }
    fn recalculate(&mut self, bounds: Rect) {
        match self {
            &mut WindowLayout::Leaf(ref mut win_) => { 
                let win = Rc::get_mut(win_).unwrap();
                win.area.x = bounds.x;
                win.area.y = bounds.y;
                win.event(Event::Resize(0,0,Point::xy(bounds.w, bounds.h)));
            },
            &mut WindowLayout::Split { children: (ref mut left, ref mut right), orientation, bounds: ref mut area, percent } => {
                let (farea, sarea) = match orientation {
                    true => (Rect::xywh(area.x, area.y, area.w*percent, area.h),
                    Rect::xywh(area.x+area.w*percent, area.y, area.w*(1.0-percent), area.h)),
                    false => (Rect::xywh(area.x, area.y, area.w, area.h*percent),
                    Rect::xywh(area.x, area.y+area.h*percent, area.w, area.h*(1.0-percent))),
                    _ => panic!()
                };
                *area = bounds;
                left.recalculate(farea);
                right.recalculate(sarea);
            }
        }
    }
    fn split_active(&mut self, ori: bool, win: Rc<Window>) {
        match self {
            &mut WindowLayout::Split { children: (ref mut left, ref mut right), .. } => {
                if let box WindowLayout::Leaf(_) = *left {
                    let mut window = match *left { box WindowLayout::Leaf(ref win) => win.clone(), _ => panic!() };
                    if window.active {
                        let area = window.area;
                        *left = Box::new(WindowLayout::Split {
                            children: (Box::new(WindowLayout::Leaf(window.clone())), Box::new(WindowLayout::Leaf(win))),
                            orientation: ori, percent: 0.5, bounds: area
                        });
                        return;
                    }
                }
                else if let box WindowLayout::Leaf(_) = *right {
                    let mut window = match *right { box WindowLayout::Leaf(ref win) => win.clone(), _ => panic!() };
                    if window.active {
                        let area = window.area;
                        *right = Box::new(WindowLayout::Split {
                            children: (Box::new(WindowLayout::Leaf(window.clone())), Box::new(WindowLayout::Leaf(win))),
                            orientation: ori, percent: 0.5, bounds: area
                        });
                        return;
                    }
                }
                {
                    left.split_active(ori, win.clone());
                    right.split_active(ori, win.clone());
                }
            },
            _ => {}
        }
    }
    fn event(&mut self, e: Event) -> bool {
        match *self {
            WindowLayout::Leaf(ref mut win) => Rc::get_mut(win).unwrap().event(e),
            WindowLayout::Split { children: (ref mut left, ref mut right), .. } => {
                left.event(e) ||
                right.event(e)
            }
        }
    }
    fn paint(&self, rx: &mut RenderContext, res: &Resources) {
        match self {
            &WindowLayout::Leaf(ref win) => win.paint(rx, res),
            &WindowLayout::Split { children: (ref left, ref right), orientation, bounds, percent } => {
                left.paint(rx, res);
                right.paint(rx, res);
                if orientation {
                    rx.draw_line(Point::xy(bounds.x+bounds.w*percent, bounds.y), Point::xy(bounds.x+bounds.w*percent, bounds.y+bounds.h), Color::rgb(0.0, 0.2, 1.0), 2.0);
                } else {
                    rx.draw_line(Point::xy(bounds.x, bounds.y+bounds.h*percent), Point::xy(bounds.x+bounds.w, bounds.y+bounds.h*percent), Color::rgb(0.0, 1.0, 0.2), 2.0);
                }
            }
        }
    }
}

struct TxdApp {
    layout: WindowLayout,
    res: Resources,
    ctrl_down: bool, ctrl_key: Option<char>
}

impl TxdApp {
    fn init(rx: &mut RenderContext) -> TxdApp {
        let mut app = TxdApp { 
            ctrl_down: false, ctrl_key: None,
            res: Resources::init(rx).expect("load resources"),
            layout: WindowLayout::Split { children: (Box::new(WindowLayout::Leaf(Rc::new(Window{ active: true, content: Box::new(TestInterface), area: rx.bounds() }))),
                    Box::new(WindowLayout::Leaf(Rc::new(Window{ active: false, content: Box::new(TestInterface), area: rx.bounds() })))), orientation: true, percent: 0.5, bounds: rx.bounds() }
        };
        app.layout.recalculate(rx.bounds());
        app
    }
}

impl App for TxdApp {
    fn event(&mut self, e: Event) {
        if let Event::Resize(_,_,size) = e {
            self.layout.recalculate(Rect::xywh(0.0,0.0,size.x,size.y));
            return;
        }
        self.layout.event(e);
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
                                    'S' => {
                                        self.layout.split_active(false, Rc::new(Window::new(Box::new(TestInterface))));
                                        let b = self.layout.bounds();
                                        self.layout.recalculate(b);
                                    } // split current window vertically
                                    'E' => {
                                        self.layout.split_active(true, Rc::new(Window::new(Box::new(TestInterface))));
                                        let b = self.layout.bounds();
                                        self.layout.recalculate(b);
                                    } // split current window horizontally
                                    'H' => {} // move left
                                    'J' => {} // move down
                                    'K' => {} // move up
                                    'L' => {} // move right
                                    _ => {}
                                },
                                _ => {}
                            }
                        } else {
                            println!("ctl {} 1", c);
                            self.ctrl_key = Some(c);
                        } 
                    } else {}
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, rx: &mut RenderContext) {
        rx.clear(Color::rgb(0.1, 0.1, 0.1));
        self.layout.paint(rx, &self.res);
    }
}

fn main() {
    SystemWindow::new("txd", 640, 480, TxdApp::init).expect("create window!").show();
}
