Loading data
============

Directly loading files
----------------------

You can directly load a supported waveform file using

```
$ viow -c 10 -t ns foo.vcd
```

This will read in all signals from foo.vcd and sample each signals value at
times 0 ns, 10 ns, 20 ns, ... 


Using a Lua script
------------------

Alternatively, you can write a Lua script to store this configuration and
configure how the data should be presented.

```lua
-- foo.lua

wave = load_vcd("foo.vcd", 10, "ns")

signals = {
	"signal_a",
	"uut.signal_b",
	"uut.macro.signal_c",
}

wave = filter_signals(wave, signals)
```

Place this into a file `foo.lua` in the same directory as `foo.vcd`. Then, you
can run

```
$ viow foo.lua
```

The script is using the `filter_signals` processing module to implement a signal
list. Only the named signals will be displayed. You can still interactively
modify this list from within viow.


More processing modules
-----------------------

Some other useful processing modules are:

1. `grep(wave, [[.*uut.*]])` to filter signals using a regular expression.
2. `replace_prefix(wave, 'some.super.deep.hierarchy.a.b.c', 'top.')` to rename
   matching prefixes of signals.
3. `remove_comments(wave)` to remove comment entries in the waveform's signal
   list.


Key bindings
============

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

In insert mode you type a partial signal name that is searched in the source
file (ignoring any filter lists if present). viow will present a list of
matching signals if any. Use Tab to cycle through them and add them to the
waveform list by pressing Enter.

Special keys:

- `ESC`: Leave insert mode.
- `TAB`: Cycle through suggestions.
- `BACKSPACE`: Delete last character.
- `ENTER`: Add selected signal to signal list.

