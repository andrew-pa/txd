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
 * + Copy/Paste (Ctrl-C/Ctrl-V and y/p) + registers
 * + Search (with regex) might be good; '/' command
 * + :s ed command?
 * + Language Server Protocol
 *   + low-level client
 *   + callbacks/tie-ins
 *   + syntax highlighting!
 *   + ensure it works/can be configured right with several different servers
 */

mod buffer;
mod mode;
mod res;
mod app;
mod movement;

use runic::{App, Window as SystemWindow};
use app::*;

fn main() {
    SystemWindow::new("txd", 1280, 640, TxdApp::init).expect("create window!").show();
}
