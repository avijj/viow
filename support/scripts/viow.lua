function help ()
	print [[
Normal mode
-----------
- `q`: Quit viow.
- `h, j, k, l`: Vi like movement of the cursor.
- `H, J, K, L`: Capital versions jump a page at a time.
- `w/b`: Jump to next/previous transition of signal under cursor.
- `+/-`: Zoom in/out on the temporal grid. Initially, one time-step is presented
  as one character wide. When zooming, with is doubled/halfed.
- `:`: Enter a Lua command in the prompt at the bottom.
- `i`: Enter insert mode before current cursor position.
- `t`: Toggle between value representations of current signal under cursor.

Insert mode
-----------
- `ESC`: Leave insert mode.
- `TAB`: Cycle through suggestions.
- `BACKSPACE`: Delete last character.
- `ENTER`: Add selected signal to signal list.

 Press Enter to continue.
	]]

	io.read()
end
