use crate::file;
use crossterm::event::KeyCode;
use crossterm::style::{Attribute, ResetColor, SetAttribute};
use crossterm::terminal;
use crossterm::{cursor, QueueableCommand};
use std::borrow::Cow;
use std::cmp;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{stdout, Write};
use std::process::Command;

#[derive(Clone)]
pub struct AppState {
    pub curr_absolute_path: String,
    pub inner_paths: Vec<file::FileData>,
    pub displayed_paths: Vec<file::FileData>,
    pub selected_index: usize,
    pub search_term: String,
    pub message: String,
}

impl AppState {
    pub fn display(&self) -> Result<(), Box<dyn Error>> {
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

        print!("{}", self.search_term);
        stdout.flush()?;
        Ok(())
    }

    pub fn handle_search_term_change(&mut self, to_push: char) {
        self.search_term.push(to_push);
        self.displayed_paths =
            file::filter_file_data(Cow::Borrowed(&self.inner_paths), &self.search_term);
    }

    pub fn handle_backspace(&mut self) {
        self.search_term.pop();
        self.displayed_paths =
            file::filter_file_data(Cow::Borrowed(&self.inner_paths), &self.search_term);
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

        let paths_data = file::get_paths(&next_dir);
        let formatted_paths = file::generate_file_data(paths_data);

        if let Ok(value) = formatted_paths {
            self.curr_absolute_path = next_dir;
            self.inner_paths = value.clone();
            self.displayed_paths = value;
            self.selected_index = 0;
            self.search_term = "".to_owned();
            self.message = "".to_owned();
        } else if let Err(e) = formatted_paths {
            self.message = e.to_string();
            return;
        }
    }

    pub fn handle_enter(&mut self) {
        let selected = &self.displayed_paths[self.selected_index];
        let curr_dir = &self.curr_absolute_path;

        std::env::set_current_dir(&curr_dir)
            .expect("HANDLE ENTER ERR: Failed to set current directory");
        if selected.shortname.contains(".") {
            if let Err(e) = Command::new("nvim").arg(&selected.shortname).status() {
                self.message = e.to_string();
                return;
            }
        }

        let paths = file::get_paths(&selected.absolute);
        let final_paths = file::generate_file_data(paths);

        if let Ok(value) = final_paths {
            self.curr_absolute_path = selected.absolute.clone();
            self.inner_paths = value.clone();
            self.displayed_paths = value;
            self.selected_index = 0;
            self.search_term = "".to_owned();
            self.message = "".to_owned();
        } else if let Err(e) = final_paths {
            self.message = e.to_string();
            return;
        }
    }

    pub fn handle_create(&mut self) {
        // the slash needs to be at the end
        // initially will only support creation in current dir
        if self.search_term.contains("/") {
            if let Err(e) = fs::create_dir(&self.search_term) {
                self.message = e.to_string();
                return;
            }
        } else if self.search_term.contains(".") {
            if let Err(e) = fs::File::create(self.search_term.clone()) {
                self.message = e.to_string();
                return;
            }
        }

        let paths = file::get_paths(&self.curr_absolute_path);
        let updated_file_data_res = file::generate_file_data(paths);
        if let Ok(value) = updated_file_data_res {
            self.search_term = "".to_owned();
            self.inner_paths = value.clone();
            self.displayed_paths = value.clone();
            self.message = String::from("File successfully created");
        } else if let Err(e) = updated_file_data_res {
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
        curr_absolute_path: cwd.to_owned(),
        inner_paths: formatted_paths.clone(),
        displayed_paths: formatted_paths.clone(),
        selected_index: 0,
        search_term: "".to_owned(),
        message: "".to_owned(),
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
            };
            test_file_data.push(fd);
        }

        return test_file_data;
    }

    #[test]
    fn test_handle_search_term_change() {
        let test_file_names: Vec<&str> = vec!["test.txt", "aaaatea.txt", "tomb.txt", "wow", "damn"];

        let test_file_data = gen_test_file_data(test_file_names);

        let mut app_state = AppState {
            curr_absolute_path: "/Test/test_dir/".to_owned(),
            inner_paths: test_file_data.clone(),
            displayed_paths: test_file_data,
            selected_index: 0,
            search_term: "".to_owned(),
            message: "".to_owned(),
        };

        struct TestCase {
            input_char: char,
            expected_search_term: &'static str,
            expected_file_names: Vec<&'static str>,
        }

        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input_char: 't',
                expected_search_term: "t",
                expected_file_names: vec!["test.txt", "tomb.txt", "aaaatea.txt"],
            },
            TestCase {
                input_char: 'e',
                expected_search_term: "te",
                expected_file_names: vec!["test.txt", "aaaatea.txt"],
            },
            TestCase {
                input_char: '.',
                expected_search_term: "te.",
                expected_file_names: vec![],
            },
        ];

        for (i, test_case) in test_cases.iter().enumerate() {
            app_state.handle_search_term_change(test_case.input_char);
            println!("{} {}", i, app_state.search_term);
            assert_eq!(
                test_case.expected_search_term.to_owned(),
                app_state.search_term
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
            curr_absolute_path: "/Test/test_dir/".to_owned(),
            inner_paths: test_file_data.clone(),
            displayed_paths: test_file_data,
            selected_index: 0,
            search_term: "test".to_owned(),
            message: "".to_owned(),
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
            assert_eq!(test_case.expected_term, app_state.search_term);
            assert_eq!(test_case.expected_file_names, file_names);
        }
    }
}
