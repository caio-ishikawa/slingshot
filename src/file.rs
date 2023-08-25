use crate::styles;
use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal;
use crossterm::{cursor, QueueableCommand};
use std::borrow::Cow;
use std::cmp;
use std::error::Error;
use std::fs;

#[derive(Clone, Debug)]
pub struct FileData {
    pub shortname: String,
    pub absolute: String,
    pub icon: String,
    pub marked_for_deletion: bool,
}

impl FileData {
    pub fn toggle_mark_for_deletion(&mut self) {
        self.marked_for_deletion = !self.marked_for_deletion;
    }
}

pub fn get_paths(source: &str) -> fs::ReadDir {
    let paths = fs::read_dir(source).expect("Could not find paths");
    return paths;
}

pub fn generate_file_data(paths: fs::ReadDir) -> Result<Vec<FileData>, Box<dyn Error>> {
    let mut output: Vec<FileData> = Vec::new();
    for path_result in paths {
        let path = path_result.expect("Failed to get DirEntry from path");
        let path_str = path.path().display().to_string();

        let icon = match_icon(&path_str);

        let split: Vec<&str> = path_str.split("/").collect();
        if let Some(last_index) = split.last() {
            let file_data = FileData {
                shortname: last_index.to_owned().to_owned(),
                absolute: path_str.clone(),
                icon,
                marked_for_deletion: false,
            };

            output.push(file_data);
        }
    }
    Ok(output)
}

fn match_icon(path: &str) -> String {
    if !path.contains(".") {
        return styles::FOLDER_ICON.to_owned();
    }

    let split_path: Vec<&str> = path.split(".").collect();
    if split_path[0] == "." || split_path[split_path.len() - 1] == "." {
        return styles::FILE_ICON.to_owned();
    }

    let mut dot_index = 0;

    for (i, item) in split_path.iter().enumerate() {
        if item == &"." {
            dot_index = i;
        }
    }

    if styles::ICONS.contains_key(split_path[dot_index + 1]) {
        return styles::ICONS[split_path[dot_index + 1]].to_owned();
    }

    return styles::FILE_ICON.to_owned();
}

pub fn filter_file_data(files: Cow<Vec<FileData>>, search_term: &str) -> Vec<FileData> {
    let mut output: Vec<FileData> = files
        .into_owned()
        .into_iter()
        .filter(|fd| {
            fd.shortname
                .to_lowercase()
                .contains(&search_term.to_lowercase())
        })
        .collect();

    output.sort_by(|a, b| {
        let cleaned_a_shortname = a.shortname.to_lowercase();
        let cleaned_b_shortname = b.shortname.to_lowercase();
        let cleaned_search_term = search_term.to_lowercase();

        let a_pos = cleaned_a_shortname.find(&cleaned_search_term);
        let b_pos = cleaned_b_shortname.find(&cleaned_search_term);

        match (a_pos, b_pos) {
            (Some(a_idx), Some(b_idx)) => a_idx.cmp(&b_idx),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => cleaned_a_shortname.cmp(&cleaned_b_shortname),
        }
    });
    output
}

pub fn print_file_data(paths: Cow<Vec<FileData>>, index: usize, stdout: &mut std::io::Stdout) {
    let mut line_index = 1;
    let (_, height) = terminal::size().unwrap();
    let t_height = cmp::max(height, 1) - 1;
    let height_limit = t_height - 1;

    stdout.queue(cursor::MoveToRow(1)).unwrap();
    for (i, path) in paths.iter().enumerate() {
        stdout.queue(cursor::MoveToRow(line_index)).unwrap();
        if i == index && i <= height_limit as usize {
            print!("{}{}{}", SetAttribute(Attribute::Bold), i, ResetColor);

            stdout.queue(cursor::MoveToColumn(3)).unwrap();
            print!("{}{}", path.icon, ResetColor);

            let fg_color = if path.marked_for_deletion {
                styles::ERR
            } else {
                styles::DEFAULT
            };

            print!(
                "{}{}{}{}",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(fg_color),
                path.shortname,
                ResetColor
            );
        } else if path.marked_for_deletion && i <= height_limit as usize {
            print!("{}{}{}", SetAttribute(Attribute::Bold), i, ResetColor);

            stdout.queue(cursor::MoveToColumn(3)).unwrap();
            print!("{}{}", path.icon, ResetColor);
            print!(
                "{}{}{}",
                SetForegroundColor(styles::ERR),
                path.shortname,
                ResetColor
            );
        } else if i <= height_limit as usize {
            print!("{}{}", SetForegroundColor(styles::LIGHT_CONTRAST), i);

            stdout.queue(cursor::MoveToColumn(3)).unwrap();
            print!("{}", path.icon);
            print!(
                "{}{}{}",
                SetForegroundColor(styles::LIGHT_CONTRAST),
                path.shortname,
                ResetColor
            );
        }
        line_index += 1;
        stdout.queue(cursor::MoveToColumn(0)).unwrap();
    }
}

#[cfg(test)]
mod file_tests {
    use super::*;

    #[test]
    fn test_filter_file_data() {
        let mut test_file_input: Vec<FileData> = Vec::new();
        let shortnames: Vec<&str> = vec!["abaaaa", "baaaa", "aaaba", "asdasddasdaasa", "bbbbbba"];

        for shortname in shortnames {
            let file_data = FileData {
                shortname: shortname.to_owned(),
                absolute: "test-absolute".to_owned(),
                icon: "test-icon".to_owned(),
                marked_for_deletion: false,
            };
            test_file_input.push(file_data);
        }

        struct TestCase {
            input: String,
            expected: Vec<&'static str>,
        }

        let test_cases = vec![
            TestCase {
                input: String::from("b"),
                expected: vec!["baaaa", "bbbbbba", "abaaaa", "aaaba"],
            },
            TestCase {
                input: String::from("bb"),
                expected: vec!["bbbbbba"],
            },
            TestCase {
                input: String::from("asd"),
                expected: vec!["asdasddasdaasa"],
            },
        ];

        for test_case in test_cases {
            let filtered = filter_file_data(Cow::Borrowed(&test_file_input), &test_case.input);

            for (i, file_data) in filtered.iter().enumerate() {
                println!("{}, ", file_data.shortname);
                assert_eq!(file_data.shortname, test_case.expected[i].to_owned())
            }
        }
    }
}
