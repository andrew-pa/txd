
use std::path::{Path, PathBuf};
use std::fs::*;
use std::io::{Read, Write, Error as IoError, ErrorKind};

pub struct Buffer {
    pub fs_loc: Option<PathBuf>,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer { fs_loc: None, lines: vec![String::from("")] }
    }

    pub fn load(fp: &Path) -> Result<Buffer, IoError> {
        let fp_exists = fp.exists();
        let mut f = OpenOptions::new().read(true).write(true).open(fp)?;
        let lns = if fp_exists { 
            let mut s : String = String::new();
            f.read_to_string(&mut s)?;
            s.lines().map(String::from).collect()
        } else {
            vec![String::from("")]
        };
        let mut buf = Buffer {
            fs_loc: Some(PathBuf::from(fp)),
            lines: lns,
        };
        Ok(buf)
    }

    pub fn insert_char(&mut self, loc: (usize,usize), c: char) {
        self.lines[loc.1].insert(loc.0, c);
    }
    pub fn delete_char(&mut self, loc: (usize, usize)) {
        self.lines[loc.1].remove(loc.0);
    }
    pub fn break_line(&mut self, loc: (usize, usize)) {
        let new_line = if loc.0 >= self.lines[loc.1].len() {
            String::from("")
        } else {
            self.lines[loc.1].split_off(loc.0)
        };
        self.lines.insert(loc.1+1, new_line);
    }

    pub fn sync_disk(&mut self) -> Result<(), IoError> {
        let lines = self.lines.iter();
        match self.fs_loc {
            Some(ref path) => {
                let mut f = OpenOptions::new().write(true).truncate(true).create(true).open(path.as_path())?;
                for ln in lines {
                    write!(f, "{}\n", ln)?;
                }
                f.sync_all()?;
                Ok(())
            },
            None => Err(IoError::new(ErrorKind::NotFound, "sync_disk with no file backing"))
        }
    }
}
