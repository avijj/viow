viow is a cycle-based, programmable waveform viewer for the terminal inspired by
vi and raw photo processing software.

!(viow screenshot)[docs/viow_screenshot.png]

Waveforms, such as Value Change Dump (VCD) files, are typically generated during
simulation of digital hardware. They record the temporal evolution of a set of
signals inside the design. Logic designers and verification engineers look at
this data to debug the design, usually after a testbench has flagged a
simulation run as failing a test.

Cycle-based means viow presents data on a fixed temporal grid. For synchronous
digital logic designs, the time-step of the grid would be set to a multiple of
the clock frequency. When opening a file format, that is not also cycle-based,
viow samples the data to a specified time-step.

viow can be extended through Lua programs. This is in particular used to
configure the processing pipeline that determines how the raw data should be
presented to the user. Similar to raw photo development software, a source file
provides raw data, that is then passed through a chain of processing modules
that can filter, edit, and add signals. The final result is then presented to
the user. Example use cases would be rewriting signal name prefixes, showing
only a subset of signals, or computing the difference between two waveforms.


(Planned) Features
==================

As of the initial version 0.1.0 viow is at a minimal demonstration level with
regards to feature completeness. The following list outlines the general goals
and motivations behind the project to give an idea where the journey is going.

 - Efficiently navigate waveform data.
 - Use Unicode characters to present data in a well readable way.
 - Interactively explore waveform data.
 - Copy and paste waveform data from the terminal to share in email or chat.
 - Automate typical debug tasks using scripts, e.g. jump to time of an error
   reported in a trace file.
 - Have rich library of processing modules to work with waveform data.
 - Support multiple file formats
   - VCD (only format supported so far)
   - GHDL Waveform (GHW) [1]
   - ...
 - Be fully configurable and extendible.



Installation
============

viow is written in Rust. To compile just run

```
$ cargo build --release
```

from within the project directory and find the binary at `target/release/viow`.

You can also install using cargo:

```
$ cargo install -- path .
```

To install Rust itself, please refer to
[1][https://www.rust-lang.org/tools/install] or your distributions package
manager.

viow is using non-ASCII Unicode characters. So, the visual result may be
different depending on what font you use in your terminal emulator. The font
[Hack][2] is known to give the expected results. Of course, your terminal
emulator also needs to support Unicode.


Configuration
=============

viow will take its configuration directory from the following sources:

1. From environment variable `VIOW_CONFIG_HOME` if set.
2. Else, from `XDG_CONFIG_HOME` if set, extended to `$XDG_CONFIG_HOME/viow`.
3. Else, from `HOME` if set, extended to `$HOME/.config/viow`.

If the resulting path is not an existing directory, no configuration is
available.

The only use for the configuration directory at the moment is to hold a
`scripts` subdirectory. This directory is added to the search path of the
embedded Lua interpreter. Load them using `require('foo')`.


Documentation
=============

See the [docs subdirectory][docs/], in particular the [usage
guide][docs/usage.md] for how to work with viow.

Some example waveform and Lua files are given in the [examples
directory][examples]

[1]: https://ghdl.github.io/ghdl/ghw/index.html
[2]: https://sourcefoundry.org/hack/
