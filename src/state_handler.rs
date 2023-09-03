use crate::file;
use crate::styles;
use crossterm::event::KeyCode;
use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal;
use crossterm::{cursor, QueueableCommand};
use std::borrow::Cow;
use std::cmp;
use std::env;
use std::error::Error;
use std::fs::{self, metadata};
use std::io::{stdout, Write};
use std::process::Command;

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
    pub fn display(&self) -> Result<(), Box<dyn Error>> {
        match self.app_mode {
            AppMode::FileExplorer => {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                let mut stdout = stdout();
                stdout.queue(cursor::MoveTo(0, 1))?;

                file::print_file_data(
                    Cow::Borrowed(&self.displayed_paths),
                    self.selected_index,
                    &mut stdout,
                );

                let (_, height) = terminal::size()?;
                let t_height = cmp::max(height, 1) - 1;
                stdout.queue(cursor::MoveTo(0, t_height))?;
                print!("{}{}", self.message, ResetColor);

                stdout.queue(cursor::MoveTo(0, 0))?;
                print!(
                    ".{}{}/",
                    SetAttribute(Attribute::Bold),
                    self.curr_absolute_path
                );

                print!("{}", self.user_input);
                if self.keybind_mode == KeybindMode::Normal {
                    stdout.queue(cursor::MoveTo(0, (self.selected_index + 1) as u16))?;
                }
                stdout.flush()?;
            }
            AppMode::Command => {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                let mut stdout = stdout();
                stdout.queue(cursor::MoveTo(0, 0))?;
                print!(
                    "{}{}",
                    SetAttribute(Attribute::Bold),
                    self.curr_absolute_path
                );
                stdout.queue(cursor::MoveTo(0, 2))?;
                let msg_split: Vec<&str> = self.message.split("\n").collect();
                if msg_split.len() > 1 {
                    for line in msg_split {
                        println!("{}", line);
                        stdout.queue(cursor::MoveToColumn(0))?;
                    }
                } else {
                    print!("{}", self.message);
                }
                stdout.queue(cursor::MoveTo(0, 1))?;
                print!("{}{}{} ", SetForegroundColor(styles::ERR), ">", ResetColor);
                print!("{}", self.user_input);
                stdout.flush()?;
            }
        }
        Ok(())
    }

    pub fn handle_user_input_change(&mut self, to_push: char) {
        self.user_input.push(to_push);
        self.displayed_paths =
            file::filter_file_data(Cow::Borrowed(&self.inner_paths), &self.user_input);
    }

    pub fn handle_backspace(&mut self) {
        self.user_input.pop();
        self.displayed_paths =
            file::filter_file_data(Cow::Borrowed(&self.inner_paths), &self.user_input);
    }

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

    pub fn handle_move_back(&mut self) {
        let mut split_dirs: Vec<&str> = self.curr_absolute_path.split("/").collect();
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

    fn handle_enter_explorer(&mut self) {
        if self.displayed_paths.len() == 0 {
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

    fn handle_enter_command(&mut self) {
        let split: Vec<&str> = self.user_input.split(" ").collect();
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
                if stdout_msg == "".to_owned() {
                    stderr_msg.trim().to_owned()
                } else {
                    stdout_msg.trim().to_owned()
                }
            }
            Err(e) => e.to_string(),
        };

        self.user_input = String::from("");
    }

    pub fn handle_enter(&mut self) {
        match self.app_mode {
            AppMode::FileExplorer => self.handle_enter_explorer(),
            AppMode::Command => self.handle_enter_command(),
        }
    }

    pub fn handle_create(&mut self) {
        if self.user_input.contains("/") {
            if let Err(e) = fs::create_dir(&self.user_input) {
                self.message = e.to_string();
                return;
            }
        } else if self.user_input.contains(".") {
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
        return;
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
            self.user_input = "".to_owned();
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
            self.user_input = "".to_owned();
            self.message = "".to_owned();
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

    return Ok(AppState {
        app_mode: AppMode::FileExplorer,
        keybind_mode: KeybindMode::Normal,
        curr_absolute_path: cwd.to_owned(),
        inner_paths: formatted_paths.clone(),
        displayed_paths: formatted_paths.clone(),
        selected_index: 0,
        user_input: "".to_owned(),
        message: "".to_owned(),
        command_mode: false,
    });
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
