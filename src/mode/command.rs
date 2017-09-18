
use super::*;
use winit::*;
use std::rc::Rc;
use std::cell::RefCell;
use buffer::Buffer;
use std::path::Path;

#[derive(Debug)]
pub enum CommandError {
    UnknownCommand,
    InvalidCommand(Option<&'static str>)
}

impl Error for CommandError {
    fn description(&self) -> &str {
        use self::CommandError::*;
        match self {
            &UnknownCommand => "Unknown command",
            &InvalidCommand(ref desc) => desc.unwrap_or("Invalid command") 
        }
    }
}

use std::fmt;
impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::*;
        match self {
            _ => write!(f, "{}", self.description())
        } 
    }
}

pub struct CommandMode {
    inserter: InsertMode
}

impl CommandMode {
    pub fn new(app: &mut app::State) -> CommandMode {
        app.bufs[0].borrow_mut().show_cursor = true;
        CommandMode { inserter: InsertMode::new_with_target(0) }
    }

    pub fn execute(&self, app: &mut app::State) -> Result<Option<Box<Mode>>, Box<Error>> {
        let cmd = { 
            app.bufs[0].borrow().lines.last().unwrap().clone()
        };
        match cmd.chars().next() {
            Some('q') => { app.should_quit = true; Ok(Some(Box::new(NormalMode::new()))) },
            Some('w') => {
                app.mutate_buf(|b| b.sync_disk())?;
                Ok(Some(Box::new(NormalMode::new())))
            },
            Some('e') => {
                let (e, path) = cmd.split_at(1);
                app.bufs.push(Rc::new(RefCell::new(Buffer::load(Path::new(path.trim()), app.res.clone())?)));
                let ix = app.bufs.len()-1;
                app.move_to_buffer(ix);
                Ok(Some(Box::new(NormalMode::new())))
            },
            Some('b') => {
                let (b, num) = cmd.split_at(1);
                let ix = if num == "#" {
                    app.last_buffer
                } else {
                    num.trim().parse::<usize>()?
                };
                if ix < 1 || ix >= app.bufs.len() {
                    Err(Box::new(CommandError::InvalidCommand(Some("Invalid buffer index"))))
                } else {
                    app.move_to_buffer(ix);
                    Ok(Some(Box::new(NormalMode::new())))
                }
            },
            Some('"') => {
                println!("-- clipstacks --");
                for (r, v) in app.clipstacks.iter() {
                    println!("\"{} = {:?}", r.0, v);
                }
                Ok(Some(Box::new(NormalMode::new())))
            },
            _ => Err(Box::new(CommandError::UnknownCommand))
        }
    }
}

impl Mode for CommandMode {
    fn event(&mut self, e: WindowEvent, app: &mut app::State) -> Result<Option<Box<Mode>>, Box<Error>> {
        match e {
            WindowEvent::KeyboardInput { input: k, .. } => 
                match k.virtual_keycode {
                    Some(VirtualKeyCode::Return) => {
                        let r = self.execute(app);
                        let mut buf_ = &app.bufs[0];
                        let mut buf = buf_.borrow_mut();
                        let len = buf.lines.len();
                        buf.show_cursor = false;
                        buf.clear();
                        r
                    }
                    Some(VirtualKeyCode::Escape) => {
                        let mut buf_ = &app.bufs[0];
                        let mut buf = buf_.borrow_mut();
                        buf.show_cursor = false;
                        buf.clear();
                        Ok(Some(Box::new(NormalMode::new())))
                    }
                    _ => self.inserter.event(e, app),
                }
            _ => self.inserter.event(e, app),
        }
    }
    fn status_tag(&self) -> &str { "COMMAND" }
}
