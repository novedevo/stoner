# Stoner

Music production plugin that emulates Carl Stone's work on Stolen Car. Emulate is used very loosely here. You could consider this outsider art of a granular synth.

## Building

After installing [Rust](https://rustup.rs/), you can compile Stoner as follows:

```shell
./make_vst.sh
```

Note that this shell script only works for compiling a Windows-compatible VST3 (i.e. a renamed .DLL file), and the shell script won't run on windows unless you have cygwin or mingw or something of the sort. This plugin is for personal use, and I personally use WSL2. The original solution (search nih-plug-xtask for more details) was recompiling all deps on every change. If you'd like to build this in a different context, you most likely need your own script to make a VST3. Rust and Cargo will happily (cross)-compile to whatever platform you like.

This repository is known to work on Rust 1.64
