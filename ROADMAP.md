
# TODO
- Get basic modal text editing working
	- Status line [done; still needs: proper line height measurement, proper placement]
	- [done] Command line [needs proper placement like status line]
		- [done with the exception of fuzzy finding/tab completion] Command parsing, at least the basics [quit/open file/write file/buffer managment]
	- Get resonable UX together (ie not opening src\main.rs at load)
	+ [done; many panics tho] Error messages (Result instead of Option from Mode switch?)
	+ [done] Delete/Change in Normal mode
		- Actually get change line to work
	- Word movements [done, except e/b movements are very broken, largly doesn't quite land cursor where one expects]
	- Paragraph movements
	- Tab key working
	+ [done; fixed in runic] Proper key repeat
	- Undo
		- Move Action parse/commit code into Buffer from Normal mode to support Undo
	- Mouse support
	- Make buffer rep more reasonable
		- Add modified flag + render it
	- Configuration stuff (colors! fonts! commands?)
		- High priority because many things are blocked due to the fact that they require configuration and it's unknown how that will work
	- Copy/Paste (Ctrl-C/Ctrl-V and y/p) + registers
	- Search (with regex) might be good; '/' command
	- :s ed command?
	- indentation commands (=, <<, >>)
	- Fix panics!!!

- Language Server Protocol
	- low-level client
	- callbacks/tie-ins
	- syntax highlighting!
	- ensure it works/can be configured right with several different servers
