# PLAN #
## things that are done ##
	- Status line [done; still needs: proper line height measurement, proper placement]
	- [done] Command line [needs proper placement like status line]
		- [done with the exception of fuzzy finding/tab completion] Command parsing, at least the basics [quit/open file/write file/buffer managment]
	+ [done ± all the other problems] Get resonable UX together (ie not opening src\main.rs at load)
	+ [done; many panics tho] Error messages (Result instead of Option from Mode switch?)
	+ [done] Delete/Change in Normal mode
		- Actually get change line to work
	- Word movements [done, except e/b movements are very broken, largly doesn't quite land cursor where one expects]
	+ [done needs configuration] Tab key working
	+ [done; fixed in runic] Proper key repeat


## things that need to be done ##
	- Configuration stuff (colors! fonts! commands?)
		- High priority because many things are blocked due to the fact that they require configuration and it's unknown how that will work
	- Copy/Paste (Ctrl-C/Ctrl-V and y/p) + registers
	- Undo
		- Move Action parse/commit code into Buffer from Normal mode to support Undo
	- Paragraph movements
	- Buffer List
	- Current Directory
	- Fix viewport wrt insertion
	- Mouse support
	- open previous buffer (:b#)
	- Make buffer rep more reasonable
		- Add modified flag + render it
		- Huuuuuge files
	- Search (with regex) might be good; '/' command
	- :s ed command?
	- indentation commands (=, <<, >>)
	- auto-indentation
	- Tab completion on buffer names/file system
	- [done sorta] fix split long lines so they do normal, regular things
	- multipule windows; even just horiz layouts
	- VISUAL mode/selection
	- folds

- Language Server Protocol
	- low-level client
	- callbacks/tie-ins
	- syntax highlighting!
	- ensure it works/can be configured right with several different servers

# things I'd like #
- newspaper-like columns view
- an image viewer

