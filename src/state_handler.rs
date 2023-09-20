use std::borrow::Cow;
use std::cmp;
use std::env;
use std::error::Error;
use std::fs::{self, metadata};
use std::io::{stdout, Stdout, Write};
use std::process::Command;

use crossterm::event::KeyCode;
use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal;
use crossterm::{cursor, QueueableCommand};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::file;
use crate::styles;

#[derive(Clone, Debug, PartialEq)]
pub enum KeybindMode {
    Normal,
    Insert,
    //Visual,
    //Deletion,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppMode {
    FileExplorer,
    Command,
}

#[derive(Clone)]
pub struct AppState {
    pub app_mode: AppMode,
    pub keybind_mode: KeybindMode,
    pub curr_absolute_path: String,
    pub inner_paths: Vec<file::FileData>,
    pub displayed_paths: Vec<file::FileData>,
    pub selected_index: usize,
    pub user_input: String,
    pub message: String,
    pub command_mode: bool,
}

impl AppState {
    // Prints all the UI elements according to the appmode
    pub fn display(&self) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();
        match self.app_mode {
            AppMode::FileExplorer => {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

                let (width, height) = terminal::size()?;
                let t_height = cmp::max(height, 1);

                stdout.queue(cursor::MoveTo(0, 2))?;
                stdout.queue(cursor::Hide)?;

                let longest_file_name = file::print_file_data(
                    Cow::Borrowed(&self.displayed_paths),
                    self.selected_index,
                    &mut stdout,
                )?;

                let preview_start_pos = longest_file_name + 6;

                // todo: x_start should be 70-80% of total width
                self.display_grid(&mut stdout, 2, preview_start_pos, width, height)?;

                stdout.queue(cursor::MoveTo(0, t_height))?;
                print!("{}{}", self.message, ResetColor);

                stdout.queue(cursor::MoveTo(0, 0))?;
                print!(
                    ".{}{}/",
                    SetAttribute(Attribute::Bold),
                    self.curr_absolute_path
                );
                stdout.flush()?;

                print!("{}", self.user_input);
                if self.keybind_mode == KeybindMode::Normal {
                    stdout.queue(cursor::MoveTo(0, (self.selected_index + 1) as u16))?;
                }

                self.display_preview(&mut stdout, preview_start_pos, width, height)?;
                self.display_grid(&mut stdout, 2, preview_start_pos, width, height)?;
                stdout.flush()?;
            }
            AppMode::Command => {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

                stdout.queue(cursor::MoveTo(0, 0))?;
                print!(
                    "{}{}",
                    SetAttribute(Attribute::Bold),
                    self.curr_absolute_path
                );

                stdout.queue(cursor::MoveTo(0, 2))?;
                let msg_split: Vec<&str> = self.message.split('\n').collect();
                if msg_split.len() > 1 {
                    for line in msg_split {
                        println!("{}", line);
                        stdout.queue(cursor::MoveToColumn(0))?;
                    }
                } else {
                    print!("{}", self.message);
                }

                stdout.queue(cursor::MoveTo(0, 1))?;
                print!("{}>{} ", SetForegroundColor(styles::ERR), ResetColor);
                print!("{}", self.user_input);
                stdout.flush()?;
            }
        }
        Ok(())
    }

    fn display_grid(
        &self,
        stdout: &mut Stdout,
        y_start: u16,
        x_start: u16,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn Error>> {
        stdout.queue(cursor::MoveTo(0, y_start))?;
        let horizontal_files: String = std::iter::repeat("━")
            .take((x_start - 2) as usize)
            .collect();
        print!("┏{horizontal_files}┓");

        stdout.queue(cursor::MoveTo(x_start, y_start))?;
        let horizontal_preview: String = std::iter::repeat("━")
            .take(((width - x_start) - 2) as usize)
            .collect();
        print!("┏{horizontal_preview}┓");

        for i in y_start + 1..height {
            stdout.queue(cursor::MoveTo(0, i))?;
            print!("┃");

            stdout.queue(cursor::MoveTo(x_start - 1, i))?;
            print!("┃");

            stdout.queue(cursor::MoveTo(x_start, i))?;
            print!("┃");

            stdout.queue(cursor::MoveTo(width, i))?;
            print!("┃");
        }

        stdout.queue(cursor::MoveTo(0, height))?;
        print!("┗{horizontal_files}┛");

        stdout.queue(cursor::MoveTo(x_start, height))?;
        print!("┗{horizontal_preview}┛");
        stdout.flush()?;
        Ok(())
    }

    // Prints the preview for the currently selected file.
    // Uses a method for the FileData enum to get the contents of the file as a string based on the
    // file extension.
    fn display_preview(
        &self,
        stdout: &mut Stdout,
        horizontal_bound: u16,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut y_index = 2; //TODO: these formatting-related values should be constants.
        let padding = 2;

        match self.app_mode {
            AppMode::FileExplorer => {
                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                if let Some(curr_file) = &self.displayed_paths.get(self.selected_index) {
                    stdout.queue(cursor::MoveTo(horizontal_bound, y_index))?;
                    let (preview_str, ignore_overflow) =
                        curr_file.as_preview_string(height * (width - horizontal_bound))?;

                    if ignore_overflow {
                        stdout.queue(cursor::MoveTo(horizontal_bound, y_index + 1))?;
                        print!("{}", preview_str);
                        stdout.queue(cursor::MoveTo(horizontal_bound, y_index + 2))?;
                        stdout.flush()?;
                        return Ok(());
                    } else {
                        if let Some(syntax) = ps.find_syntax_by_extension(&curr_file.extension) {
                            // it is possible to change attributes via t.settings.<setting>. e.g.
                            // background
                            let t = ts.themes["base16-ocean.dark"].clone();
                            let mut h = HighlightLines::new(syntax, &t);

                            for line in LinesWithEndings::from(&preview_str) {
                                y_index += 1;
                                stdout
                                    .queue(cursor::MoveTo(horizontal_bound + padding, y_index))?;

                                let ranges: Vec<(Style, &str)> =
                                    h.highlight_line(line, &ps).expect("range");
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], true);

                                print!("{}{}", escaped, ResetColor);
                                stdout.flush()?;

                                if y_index == height - 3 {
                                    return Ok(());
                                }
                            }
                        } else {
                            print!("unsupported syntax");
                        }
                    }

                    stdout.queue(cursor::MoveTo(horizontal_bound, y_index + 1))?;
                    stdout.flush()?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    // Updates the user_input field and the filtered results
    pub fn handle_user_input_change(&mut self, to_push: char) {
        self.user_input.push(to_push);
        self.displayed_paths =
            file::filter_file_data(Cow::Borrowed(&self.inner_paths), &self.user_input);
    }

    // Updates the user_input field and the filtered results
    // TODO: Incorporate into handle_user_input_change
    pub fn handle_backspace(&mut self) {
        self.user_input.pop();
        self.displayed_paths =
            file::filter_file_data(Cow::Borrowed(&self.inner_paths), &self.user_input);
    }

    // Updates the selected index of the file according to pressed key.
    pub fn update_selected_index(&mut self, action: KeyCode) {
        let mut updated_index = 0;
        let total_dirs = self.displayed_paths.len() - 1;

        if action == KeyCode::Down {
            if self.selected_index == total_dirs {
                updated_index = 0;
            } else {
                updated_index = self.selected_index + 1;
            }
        } else if action == KeyCode::Up {
            if self.selected_index == 0 {
                updated_index = total_dirs
            } else {
                updated_index = self.selected_index - 1;
            }
        }

        self.selected_index = updated_index;
    }

    // Moves back one directory based on the current absolute path.
    // TODO: Use std::path::Path and std::file::File instead of the absolute path to avoid crashing
    // when moving back from the home directory.
    pub fn handle_move_back(&mut self) {
        let mut split_dirs: Vec<&str> = self.curr_absolute_path.split('/').collect();
        split_dirs.pop();

        let next_dir: String = split_dirs
            .join("/")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        if let Err(e) = std::env::set_current_dir(&next_dir) {
            self.message = e.to_string();
            return;
        }

        self.update_post_move(&next_dir);
    }

    // Handles Enter key press in FileExploerer mode.
    // Either opens a file in NeoVim or enters a new directory, updating the curernt state.
    fn handle_enter_explorer(&mut self) {
        if self.displayed_paths.is_empty() {
            self.handle_create();
            return;
        }

        let selected = &self.displayed_paths[self.selected_index];
        let metadata_res = metadata(&selected.absolute);
        match metadata_res {
            Ok(metadata) => {
                if metadata.is_file() {
                    if let Err(e) = Command::new("nvim").arg(&selected.shortname).status() {
                        self.message = e.to_string();
                        return;
                    }
                }
            }
            Err(e) => {
                self.message = e.to_string();
                return;
            }
        }

        if let Err(e) = std::env::set_current_dir(&selected.absolute) {
            self.message = e.to_string();
            return;
        }

        self.update_post_move(&selected.absolute.clone());
    }

    // Handles enter key press in Command mode.
    // Attemtps to run the command in self.user_input synchronously, and displays the output.
    fn handle_enter_command(&mut self) {
        let split: Vec<&str> = self.user_input.split(' ').collect();
        let args: Vec<&str> = split[1..].iter().map(|x| x.to_owned()).collect();
        let cmd_res = Command::new(split[0])
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .output();

        self.message = match cmd_res {
            Ok(output) => {
                let stdout_msg = String::from_utf8_lossy(&output.stdout);
                let stderr_msg = String::from_utf8_lossy(&output.stderr);
                if stdout_msg == *"" {
                    stderr_msg.trim().to_owned()
                } else {
                    stdout_msg.trim().to_owned()
                }
            }
            Err(e) => e.to_string(),
        };

        self.user_input = String::from("");
    }

    // Delegates to enter handler function depdending on current mode.
    pub fn handle_enter(&mut self) {
        match self.app_mode {
            AppMode::FileExplorer => self.handle_enter_explorer(),
            AppMode::Command => self.handle_enter_command(),
        }
    }

    // Creates file with name used in self.user_input. It distinguishes between file and folder
    // creation depending on the use of '/' and '.' characters.
    pub fn handle_create(&mut self) {
        if self.user_input.contains('/') {
            if let Err(e) = fs::create_dir(&self.user_input) {
                self.message = e.to_string();
                return;
            }
        } else if self.user_input.contains('.') {
            if let Err(e) = fs::File::create(self.user_input.clone()) {
                self.message = e.to_string();
                return;
            }
        }

        self.update_paths();
        self.message = String::from("File successfully created");
    }

    pub fn handle_mark_delete(&mut self) {
        self.displayed_paths[self.selected_index].toggle_mark_for_deletion();
        let marked_files: Vec<&str> = self
            .displayed_paths
            .iter()
            .filter(|fd| fd.marked_for_deletion)
            .map(|fd| fd.shortname.as_str())
            .collect();

        self.message = format!(
            "Press Ctrl + Y to confirm deletion of files: {:?}",
            marked_files
        );
    }

    pub fn handle_confirm_delete(&mut self) {
        for path in &self.displayed_paths {
            if path.marked_for_deletion {
                if let Err(e) = fs::remove_file(&path.absolute) {
                    self.message = e.to_string();
                    return;
                }
            }
        }

        self.update_paths();
        self.message = String::from("Files successfully removed");
    }

    pub fn toggle_command_mode(&mut self) {
        self.user_input = String::from("");
        self.message = String::from("");
        self.update_paths();

        match self.app_mode {
            AppMode::FileExplorer => self.app_mode = AppMode::Command,
            AppMode::Command => self.app_mode = AppMode::FileExplorer,
        }
    }

    pub fn handle_unsupported_input(&mut self) {
        self.message = "Unsupported input.".to_owned();
    }

    fn update_paths(&mut self) {
        let paths = file::get_paths(&self.curr_absolute_path);
        let updated_file_data_res = file::generate_file_data(paths);
        if let Ok(value) = updated_file_data_res {
            self.user_input = String::new();
            self.inner_paths = value.clone();
            self.displayed_paths = value.clone();
        } else if let Err(e) = updated_file_data_res {
            self.message = e.to_string();
            return;
        }
    }

    fn update_post_move(&mut self, absolute_path: &str) {
        let paths = file::get_paths(absolute_path);
        let final_paths = file::generate_file_data(paths);

        if let Ok(value) = final_paths {
            self.curr_absolute_path = absolute_path.to_owned();
            self.inner_paths = value.clone();
            self.displayed_paths = value;
            self.selected_index = 0;
            self.user_input = String::new();
            self.message = String::new();
        } else if let Err(e) = final_paths {
            self.message = e.to_string();
            return;
        }
    }
}

pub fn initial_app_state() -> Result<AppState, Box<dyn Error>> {
    let cwd_buf = env::current_dir()?;
    let cwd = cwd_buf.to_str().expect("Could not convert path to str");

    let paths = file::get_paths(cwd);
    let formatted_paths = file::generate_file_data(paths).unwrap();

    Ok(AppState {
        app_mode: AppMode::FileExplorer,
        keybind_mode: KeybindMode::Normal,
        curr_absolute_path: cwd.to_owned(),
        inner_paths: formatted_paths.clone(),
        displayed_paths: formatted_paths.clone(),
        selected_index: 0,
        user_input: "".to_owned(),
        message: "".to_owned(),
        command_mode: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_test_file_data(names: Vec<&str>) -> Vec<file::FileData> {
        let mut test_file_data: Vec<file::FileData> = Vec::new();
        for name in names {
            let fd = file::FileData {
                shortname: name.to_owned(),
                absolute: "".to_owned(),
                icon: "".to_owned(),
                extension: "".to_owned(),
                marked_for_deletion: false,
            };
            test_file_data.push(fd);
        }

        return test_file_data;
    }

    #[test]
    fn test_handle_user_input_change() {
        let test_file_names: Vec<&str> = vec!["test.txt", "aaaatea.txt", "tomb.txt", "wow", "damn"];

        let test_file_data = gen_test_file_data(test_file_names);

        let mut app_state = AppState {
            app_mode: AppMode::FileExplorer,
            keybind_mode: KeybindMode::Normal,
            curr_absolute_path: "/Test/test_dir/".to_owned(),
            inner_paths: test_file_data.clone(),
            displayed_paths: test_file_data,
            selected_index: 0,
            user_input: "".to_owned(),
            message: "".to_owned(),
            command_mode: false,
        };

        struct TestCase {
            input_char: char,
            expected_user_input: &'static str,
            expected_file_names: Vec<&'static str>,
        }

        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input_char: 't',
                expected_user_input: "t",
                expected_file_names: vec!["test.txt", "tomb.txt", "aaaatea.txt"],
            },
            TestCase {
                input_char: 'e',
                expected_user_input: "te",
                expected_file_names: vec!["test.txt", "aaaatea.txt"],
            },
            TestCase {
                input_char: '.',
                expected_user_input: "te.",
                expected_file_names: vec![],
            },
        ];

        for (i, test_case) in test_cases.iter().enumerate() {
            app_state.handle_user_input_change(test_case.input_char);
            println!("{} {}", i, app_state.user_input);
            assert_eq!(
                test_case.expected_user_input.to_owned(),
                app_state.user_input
            );
            let file_names: Vec<&str> = app_state
                .displayed_paths
                .iter()
                .map(|fd| fd.shortname.as_str())
                .collect();
            assert_eq!(test_case.expected_file_names, file_names)
        }
    }

    #[test]
    fn test_handle_backspace() {
        let test_file_names: Vec<&str> = vec!["test.txt", "aaaatea.txt", "tomb.txt", "wow", "damn"];
        let test_file_data = gen_test_file_data(test_file_names);

        let mut app_state = AppState {
            app_mode: AppMode::FileExplorer,
            keybind_mode: KeybindMode::Normal,
            curr_absolute_path: "/Test/test_dir/".to_owned(),
            inner_paths: test_file_data.clone(),
            displayed_paths: test_file_data,
            selected_index: 0,
            user_input: "test".to_owned(),
            message: "".to_owned(),
            command_mode: false,
        };

        struct TestCase {
            expected_term: &'static str,
            expected_file_names: Vec<&'static str>,
        }

        let test_cases: Vec<TestCase> = vec![
            TestCase {
                expected_term: "tes",
                expected_file_names: vec!["test.txt"],
            },
            TestCase {
                expected_term: "te",
                expected_file_names: vec!["test.txt", "aaaatea.txt"],
            },
            TestCase {
                expected_term: "t",
                expected_file_names: vec!["test.txt", "tomb.txt", "aaaatea.txt"],
            },
        ];

        for test_case in test_cases {
            app_state.handle_backspace();
            let file_names: Vec<&str> = app_state
                .displayed_paths
                .iter()
                .map(|fd| fd.shortname.as_str())
                .collect();
            assert_eq!(test_case.expected_term, app_state.user_input);
            assert_eq!(test_case.expected_file_names, file_names);
        }
    }
}
