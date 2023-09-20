use base64::engine::general_purpose;
use base64::Engine;

use std::borrow::Cow;
use std::cmp;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{BufReader, Read};

use crossterm::style::{Attribute, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal;
use crossterm::{cursor, QueueableCommand};

use crate::styles;

#[derive(Clone, Debug)]
pub struct FileData {
    pub shortname: String,
    pub absolute: String,
    pub extension: String,
    pub icon: String,
    pub marked_for_deletion: bool,
}

impl FileData {
    pub fn toggle_mark_for_deletion(&mut self) {
        self.marked_for_deletion = !self.marked_for_deletion;
    }

    // Returns a string used to display the file preview. It checks the file extension to determine
    // the logic to compute the return string. Unsupported file extension returns generic metadata.
    pub fn as_preview_string(&self, char_limit: u16) -> Result<(String, bool), Box<dyn Error>> {
        let image_preview_formats: [&str; 11] = [
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "svg", "ico", "psd",
        ];

        let text_preview_formats: [&str; 22] = [
            "txt", "pdf", "doc", "md", "rs", "py", "js", "ts", "svelte", "html", "hs", "ml", "c",
            "cpp", "h", "zig", "go", "json", "toml", "MAKEFILE", "Makefile", "makefile",
        ];

        match self.extension.as_str() {
            "" => Ok((String::from("directory"), false)),
            x if image_preview_formats.contains(&x) => return self.iterm_inline_img(),
            x if text_preview_formats.contains(&x) => return self.text_preview(char_limit),
            _ => Ok((String::from("Unsupported file extension"), false)),
        }
    }

    // Returns a vectory of bytes representing the content of the file.
    fn as_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let file = fs::File::open(&self.absolute)?;
        let mut reader = BufReader::new(file);
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;

        Ok(content)
    }

    // Returns an iTerm Image Protocol encoded string, which when printed, will display a picture
    // in supported terminals.
    // Read more: https://iterm2.com/documentation-images.html
    fn iterm_inline_img(&self) -> Result<(String, bool), Box<dyn Error>> {
        let encoded = general_purpose::STANDARD.encode(self.as_bytes()?);

        let iterm_command = format!(
            "\x1b]1337;File=inline=1;size={}:{}\x07",
            encoded.len() - 100,
            encoded,
        );

        Ok((iterm_command, true))
    }

    // Returns a string representing the contents of the file. The length of the string is capped
    // based on the char_limit parameter. This parameter represents half of the resolution of the
    // terminal, but does not take into account newline characters, meaning this string cannot be
    // printed directly without the overflow affecting the UI.
    fn text_preview(&self, char_limit: u16) -> Result<(String, bool), Box<dyn Error>> {
        let mut output = String::new();
        let bytes = self.as_bytes()?;
        for i in 0..char_limit {
            if let Some(byte) = bytes.get(i as usize) {
                output.push(*byte as char);
            } else {
                return Ok((output, false));
            }
        }

        Ok((output, false))
    }
}

//TODO: remove this
pub fn get_paths(source: &str) -> fs::ReadDir {
    fs::read_dir(source).expect("Could not find paths")
}

pub fn generate_file_data(paths: fs::ReadDir) -> Result<Vec<FileData>, Box<dyn Error>> {
    let mut output: Vec<FileData> = Vec::new();
    for path_result in paths {
        let path = path_result.expect("Failed to get DirEntry from path");
        let path_str = path.path().display().to_string();

        let path = std::path::Path::new(&path_str);

        let mut extension = String::new();
        let mut file_name = String::new();
        if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            extension = ext.to_owned();
        } else if path.is_file() {
            if let Some(t) = path.file_name().and_then(OsStr::to_str) {
                extension = t.to_owned();
                file_name = t.to_owned();
            } else {
                panic!("Could not get file name");
            }
        }

        if file_name.is_empty() {
            let split: Vec<&str> = path_str.split('/').collect();
            if let Some(last_index) = split.last() {
                file_name = last_index.to_owned().to_owned()
            }
        }

        let icon = if path.is_dir() {
            styles::FOLDER_ICON.to_owned()
        } else if styles::ICONS.contains_key(&extension) {
            styles::ICONS[&extension].to_owned()
        } else {
            styles::FILE_ICON.to_owned()
        };

        let file_data = FileData {
            shortname: file_name,
            absolute: path_str.clone(),
            extension,
            icon,
            marked_for_deletion: false,
        };

        output.push(file_data);
    }

    Ok(output)
}

pub fn filter_file_data(files: Cow<Vec<FileData>>, search_term: &str) -> Vec<FileData> {
    let mut output: Vec<FileData> = files
        .into_owned()
        .iter()
        .cloned()
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

// Prints file data and returns length of longest file name, to determine the horizontal bounds for
// the file preview
pub fn print_file_data(
    paths: Cow<Vec<FileData>>,
    index: usize,
    stdout: &mut std::io::Stdout,
) -> Result<u16, Box<dyn Error>> {
    let mut line_index = 2;
    let (_, height) = terminal::size()?;
    let t_height = cmp::max(height, 1);
    let height_limit = t_height - 1;
    let mut longest_file_name: u16 = 0;

    stdout.queue(cursor::MoveToRow(1))?;
    for (i, path) in paths.iter().enumerate() {
        let count = path.shortname.chars().count() as u16;
        if count > longest_file_name {
            longest_file_name = count;
        }
        stdout.queue(cursor::MoveToRow(line_index))?;
        if i == index && i <= height_limit as usize {
            print!("  {}{}", path.icon, ResetColor);

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
            print!("  {}{}", path.icon, ResetColor);
            print!(
                "{}{}{}",
                SetForegroundColor(styles::ERR),
                path.shortname,
                ResetColor
            );
        } else if i <= height_limit as usize {
            print!("  {}", path.icon);
            print!(
                "{}{}{}",
                SetForegroundColor(styles::LIGHT_CONTRAST),
                path.shortname,
                ResetColor
            );
        }
        line_index += 1;
        stdout.queue(cursor::MoveToColumn(0))?;
    }

    Ok(longest_file_name)
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
                extension: "py".to_owned(),
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
