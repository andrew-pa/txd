extern crate runic;
// txd: a text editor🖳

/* TODO
 * + Get basic modal text editing working
 *   + Status line [partial; still needs: proper line height measurement, proper placement]
 *   + Command line [rendering seems good except status line problems as above]
 *     + Good command parsing, at least the basics [quit/open file/write file/buffer managment]
 *   + Get resonable UX together (ie not opening src\main.rs at load)
 *   + Error messages (Result instead of Option from Mode switch?)
 * + Make buffer rep more reasonable
 * + Configuration stuff (colors! fonts! commands?)
 * + Search (with regex) might be good; '/' command
 * + :s ed command?
 * + Language Server Protocol
 *   + low-level client
 *   + callbacks/tie-ins
 *   + syntax highlighting!
 *   + ensure it works/can be configured right with several different servers
 */

mod buffer;
mod bufferview;
mod mode;
mod res;
mod app;
mod movement;

use runic::{App, Window as SystemWindow};
use app::*;

struct TestObj;

impl Drop for TestObj {
    fn drop(&mut self) {
        println!("drop");
    }
}

fn main() {
    let to = TestObj;
    SystemWindow::new("txd", 1280, 640, TxdApp::init).expect("create window!").show();
}
