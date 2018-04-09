use std::error::Error;
use std::fs::File;
use std::io::{Read, ErrorKind as IOErrorKind};
use runic::*;

use toml::Value;

pub struct Resources {
    pub config: Option<Value>,
    pub font: Font
}

impl Resources {
    pub fn new(mut rx: &mut RenderContext, config: Option<Value>) -> Result<Resources, Box<Error>> {
        let font = config.as_ref().and_then(|c| c.get("font"));
        let font_name = font.and_then(|f| f.get("name").and_then(Value::as_str)).unwrap_or("Consolas");
        let font_size = font.and_then(|f| f.get("size").and_then(Value::as_float)).unwrap_or(14.0);
        Ok(Resources {
            config: config.clone(),
            font: rx.new_font(font_name, font_size as f32, FontWeight::Regular, FontStyle::Normal)?,
        })
    }
}
