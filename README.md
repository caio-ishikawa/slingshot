slingshot 0.1.0
============

[![Tests](https://github.com/caio-ishikawa/slingshot/actions/workflows/build.yml/badge.svg?branch=master)](https://github.com/caio-ishikawa/slingshot/actions/workflows/build.yml)

<img src="https://s11.gifyu.com/images/ScefB.gif">

Slingshot is a lightweight tool to browse files in the terminal. It allows the user to quickly filter through files in any directory, open them with a text editor (nvim by default), and create/edit/delete files in a simple interface.

Design Goals
------------
- A quick way to navigate, create and edit files in the terminal.
- Easily maintanable.
- Minimal use of third party crates.

Dependencies
------------
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- [Nerdfonts](https://www.nerdfonts.com/)

How to install
--------------
- Clone the repository & navigate to cloned directory.
- Run `make build`
- Run `make install`

Known issues
------------
- Scrolling/overflows do not render properly (filtering is not affected).
- Crashes if user tries to move back from home directory.

