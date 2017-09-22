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
	+ [done] open previous buffer (:b#)
	+ [done] Fix viewport wrt insertion
	+ [done] y/p/P commands + clipstacks


## things that need to be done ##
	- Configuration stuff (colors! fonts! commands?)
		- High priority because many things are blocked due to the fact that they require configuration and it's unknown how that will work
	- Copy/Paste (Ctrl-C/Ctrl-V)
		- add perhaps a way to index into the stack, also move between them (dup/swap?)
	- Undo
		- Move Action parse/commit code into Buffer from Normal mode to support Undo
		- Make insertion an Action that actually holds the inserted content???
	- Paragraph movements
	- Buffer List
	- Current Directory
	- Mouse support
	- Tab completion on buffer names/file system
	- don't reload already open files into a new buffer
	- Make buffer rep more reasonable
		- Add modified flag + render it
		- Huuuuuge files
	- Search (with regex) might be good; '/' command
	- :s ed command?
		- ed/ex commands!
	- indentation commands (=, <<, >>)
	- auto-indentation
	- [done sorta] fix split long lines so they do normal, regular things
	- multipule windows; even just horiz layouts
	- VISUAL mode/selection
	- folds
	- syntax where the rep count comes before the action -> 3dw instead of d3w
	- resizing the window should change the line wrap
	- command output, somewhere
	- inclusive/exclusive/linewise motions like Vim
	- close buffers

- Language Server Protocol
	- low-level client
	- callbacks/tie-ins
	- syntax highlighting!
	- ensure it works/can be configured right with several different servers

# things I'd like #
- newspaper-like columns view
- an image viewer

