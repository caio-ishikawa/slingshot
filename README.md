slingshot 0.2.1
===============

[![Tests](https://github.com/caio-ishikawa/slingshot/actions/workflows/build.yml/badge.svg?branch=master)](https://github.com/caio-ishikawa/slingshot/actions/workflows/build.yml)

<img src="https://i.imgur.com/Psberkp.gif">

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

How to use
----------
- Moving up/down:
    - Arrow Keys
    - `Ctrl+J`/`Ctrl+K`

- Creating folders/files:
    - For folders, type the desired name followed by a `/`.
    - For files, type the name of the desired file with the file extension (e.g. `.py`, `.txt`, etc.)
    - Confirm creation by pressing Enter.

- Deleting files/folders:
    - Marking files/folders for deletion is done by pressing Ctrl+D, which will highlight the item red.
    - Confirm by pressing `Ctrl+Y`.

- Command mode:
    - Toggling between Command Mode and File Explorer can be done by pressing `Ctrl+N`.
    - To run the command, type it and confirm with Enter.

Known issues
------------
- Scrolling/overflows do not render properly (filtering is not affected.)
- Crashes if user tries to move back from home directory.
- Cursor does not move correctly in command mode (functionality not affected.)

