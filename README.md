terminfo-lean
=============
[![Build Status](https://github.com/proski/terminfo-lean/actions/workflows/rust.yml/badge.svg)](https://github.com/proski/terminfo-lean/actions/workflows/rust.yml)

Rust crate that provides access to terminal capabilities from the terminfo database.

This is a fork of the terminfo crate intended to be simple and lightweight.

Licensing
---------

terminfo-lean is licensed under Apache License, Version 2.0. Please see
([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

Documentation
-------------

The crate documentation will be available under <https://docs.rs/terminfo/latest/terminfo-lean/>

Example
-------

```rust
use std::io;
use terminfo_lean::{capability as cap, Database};

fn main() {
	let info = Database::from_env().unwrap();

	if let Some(cap::MaxColors(n)) = info.get::<cap::MaxColors>() {
		println!("The terminal supports {} colors.", n);
	} else {
		println!("The terminal does not support colors, what year is this?");
	}

	if let Some(flash) = info.get::<cap::FlashScreen>() {
		flash.expand().to(io::stdout()).unwrap();
	} else {
		println!("FLASH GORDON!");
	}

	info.get::<cap::SetAForeground>().unwrap().expand().color(2).to(io::stdout()).unwrap();
	info.get::<cap::SetABackground>().unwrap().expand().color(4).to(io::stdout()).unwrap();
	println!("SUP");
	info.get::<cap::ExitAttributeMode>().unwrap().expand().to(io::stdout()).unwrap();
}
```

Packaging and Distributing
--------------------------
For all terminals but windows consoles, this library depends on a non-hashed
(for now) terminfo database being present. For example, on Debian derivatives,
you should depend on ncurses-term; on Arch Linux, you depend on ncurses; and on
MinGW, you should depend on mingw32-terminfo.

Unfortunately, if you're using a non-windows console on Windows (e.g. MinGW,
Cygwin, Git Bash), you'll need to set the TERMINFO environment variable to
point to the directory containing the terminfo database.
