extern crate runic;

use runic::{App, Window as SystemWindow, Event, RenderContext, Color, Point, Rect, Font};

/*
    System-Window [
        Window [ TextBuffer ]
        Window [ Image ]
        Window [ Terminal ]
    ]

    Window [ Client-Area; Status-Bar ]
*/

trait Interface {
    fn event(&mut self, e: Event) -> bool; //handled?
    fn paint(&self, rx: &mut RenderContext);
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

    fn paint(&self, rx: &mut RenderContext) {
        rx.stroke_rect(self.area, Color::rgb(1.0, 1.0, 1.0), 2.0);
        rx.translate(Point::xy(self.area.x, self.area.y));
        self.content.paint(rx);
        rx.translate(Point::xy(0.0, 0.0));
    }
}

struct TestInterface;

impl Interface for TestInterface {
    fn event(&mut self, e: Event) -> bool { false }
    fn paint(&self, rx: &mut RenderContext) {
        rx.fill_rect(Rect::xywh(8.0, 8.0, 64.0, 64.0), Color::rgb(0.2, 0.6, 0.4));
    }
}

#[derive(Clone)]
enum WindowTree {
    Split {
       first: Box<WindowTree>, second: Box<WindowTree>,
       percent: f32, orientation: u8
    },
    Leaf(usize) //index into window list
}

struct TxdApp {
    windows: Vec<Window>,
    active_window: usize,
    layout: WindowTree
}

impl TxdApp {
    fn init(rx: &mut RenderContext) -> TxdApp {
        TxdApp { 
            windows: vec![ 
                Window{area: rx.bounds(), content: Box::new(TestInterface)},
                Window{area: rx.bounds(), content: Box::new(TestInterface)},
                Window{area: rx.bounds(), content: Box::new(TestInterface)}
            ], 
            active_window: 0, 
            layout: WindowTree::Split { 
                first: Box::new(WindowTree::Split { 
                    first: Box::new(WindowTree::Leaf(0)),
                    second: Box::new(WindowTree::Leaf(1)),
                    percent: 0.7, orientation: 1
                }), 
                second: Box::new(WindowTree::Leaf(2)), percent: 0.5, orientation: 0 }
        }
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
    }

    fn paint(&self, rx: &mut RenderContext) {
        for w in self.windows.iter() {
            w.paint(rx);
        }
    }
}

fn main() {
    SystemWindow::new("txd", 1920, 1080, TxdApp::init).expect("create window!").show();
}
