use std::error::Error;
use runic::*;

pub struct Resources {
    pub font: Font
}

impl Resources {
    pub fn new(mut rx: &mut RenderContext) -> Result<Resources, Box<Error>> {
        Ok(Resources {
            font: rx.new_font("Consolas", 12.0, FontWeight::Regular, FontStyle::Normal)?,
        })
    }
}
