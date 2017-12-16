
use std::io::{Error as IOError};
use std::path::{Path, PathBuf};
use std::fs;
/* one day
/// calculate the relative path to a file from inside of a directory
/// this will traverse the file system. Assumes that both dir and file are absolute paths from root
fn relative_path(dir: &Path, file_path: &Path) -> Result<PathBuf, IOError> {
    if !file_path.exists() { return Ok(PathBuf::from(file_path)); } // a file that doesn't exist stays with it's whole path
    //$cd = start in $dir, split the $file path into $path and $file with $file=the filename+ext
    let mut stack = Vec::new();
    stack.push(dir);
    let file = file_path.file_name().ok_or(IOError::new(::std::io::ErrorKind::InvalidInput, "file path does not have a file name"))?;
    loop {
    //if there is stuff on the stack: $cd = pop stack
        if stack.len() == 0 { break; }
        let cd = stack.pop();
        for _de in fs::read_dir(cd)? {
            let de = _de?;
            let ft = de.file_type()?;
            if ft.is_file() && de.file_name() == file {
                // we found $file in $cd
            }
        }

    //does $cd = $path and contain $file
    //  yes => return $cd+$file
    //  no => does $cd have any child directories?
    //      yes => push absolute children onto stack
    //      nope => push parent directory onto stack 
    }
    Ok(PathBuf::from(file))
}

#[cfg(test)]
mod tests {
    // these tests assume you are in the root repo directory
    use super::*;
    #[test]
    fn trivial_rel_path() {
        let cd = ::std::env::current_dir().unwrap();
        assert_eq!(Path::new("README.md"), relative_path(&cd, &cd.join("README.md")).unwrap());
    }
}*/
