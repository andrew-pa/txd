extern crate winapi;
extern crate kernel32;

use winapi::*;
use kernel32::*;

trait Console {
	fn move_cursor(&self, x : i16, y : i16);
}

struct Win32Console {
	screen_buffer : HANDLE
}

impl Win32Console {
	fn new() -> Win32Console {
		Win32Console {
			screen_buffer : unsafe {
				kernel32::CreateConsoleScreenBuffer(GENERIC_READ | GENERIC_WRITE, FILE_SHARE_WRITE,
					0 as *const SECURITY_ATTRIBUTES, CONSOLE_TEXTMODE_BUFFER, 0 as LPVOID)
			}
		}
	}
}

impl Console for Win32Console {
	fn move_cursor(&self, x : i16, y : i16) {
		unsafe {
			kernel32::SetConsoleCursorPosition(self.screen_buffer, COORD {X: x, Y: y});
		}
	}
}

impl Drop for Win32Console {
	fn drop(&mut self) {
		unsafe {
			kernel32::CloseHandle(self.screen_buffer);
		}
	}
}

fn main() {
	let cnsl = Win32Console::new();

	println!("hello, world!");
	let mut x = 30i16;
	loop { cnsl.move_cursor(x, 30); x += 1; if x > 80 {x = 30;} }
}
