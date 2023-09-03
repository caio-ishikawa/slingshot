slingshot 0.3.0
===============

[![Tests](https://github.com/caio-ishikawa/slingshot/actions/workflows/build.yml/badge.svg?branch=master)](https://github.com/caio-ishikawa/slingshot/actions/workflows/build.yml)

<img src="https://i.imgur.com/Psberkp.gif">

Slingshot is a lightweight tool to browse files in the terminal. It allows the user to quickly filter through files in any directory, open them with a text editor (nvim by default), create/edit/delete files , and run commands in a simple interface.

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
- Build from source:
    - Clone the repository & navigate to cloned directory.
    - Run `make build`
    - Run `make install`

How to use
----------
Slingshot aims to closely resemble vim motions to ensure a coherent workflow. 
Once started, Slingshot defaults to `normal mode`.

Normal mode:
- Used for navigation.
- [J, K] can be used to navigate up and down the file list.
- [H, L] can be used to navigate back one directory, or to enter the selected directory.
- [I, A] can be used to switch to `insert mode`

Insert mode:
- Used for typing the search term. 
- [Enter] can be used to enter the selected file.

Global commands:
- [`Ctrl+C`] to quit application,
- [`Ctrl+N`] to run commands.

Fish Shell Integration
----------------------
The only requirement is to have slingshot installed.

1. Run `fisher install caio-ishikawa/slingshot-fish`.

The default keybind to open slingshot in the fish shell is `Ctrl+S`.

Known issues
------------
- Scrolling/overflows do not work. (filtering is not affected.)
- Crashes if user tries to move back from home directory.

