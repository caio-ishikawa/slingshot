use crate::state_handler;
use std::io::{stdout, Write};
use crossterm::event::{KeyCode, KeyModifiers};
use std::borrow::Cow;
use std::error::Error;
use crossterm::cursor;
use std::cmp;
use crossterm::execute;

pub fn handle_key_code(
    key_code: KeyCode,
    app_state: &mut state_handler::AppState,
) -> Result<state_handler::AppState, Box<dyn Error>> {
    match key_code {
        KeyCode::Char(c) => {
            app_state.handle_search_term_change(c);
            return Ok(app_state.clone());
        }
        KeyCode::Backspace => {
            app_state.handle_backspace();
            return Ok(app_state.clone());
        }
        KeyCode::Enter => {
            let new_state = app_state.handle_enter(
                Cow::Borrowed(&app_state.displayed_paths[app_state.selected_index]),
                &app_state.curr_absolute_path,
            )?;
            return Ok(new_state);
        }
        KeyCode::Up | KeyCode::Down => {
            app_state.update_selected_index(key_code);
            return Ok(app_state.clone());
        }
        KeyCode::Left => {
            let new_state = app_state.handle_move_back()?;
            return Ok(new_state);
        }
        KeyCode::Esc => panic!(),
        _ => return Ok(app_state.clone()),
    }
}

pub fn handle_key_modifier(key_code: KeyCode, modifier: KeyModifiers, app_state: &mut state_handler::AppState) -> Result<(), Box<dyn Error>> {
    if modifier == KeyModifiers::CONTROL{
        match key_code {
            KeyCode::Char('n') => {
                app_state.handle_create();
                return Ok(());
            },
            _ => panic!("not implemented")

        }
    }

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::file;
    use std::path::Path;

    fn enter_test_dir() -> state_handler::AppState {
        let test_dir_path = "tests";
        let curr_dir = std::env::current_dir().expect("Could not get current dir");
        let absolute_path = curr_dir
            .join(Path::new(test_dir_path))
            .to_str()
            .expect("Could not turn path to str")
            .to_owned();

        let paths = std::fs::read_dir(&absolute_path).expect("Could not find paths");
        let formatted_paths = file::generate_file_data(paths).expect("Error generating file data");

        return state_handler::AppState {
            curr_absolute_path: absolute_path,
            inner_paths: formatted_paths.clone(),
            displayed_paths: formatted_paths.clone(),
            selected_index: 0,
            search_term: "".to_owned(),
            message: "".to_owned()
        };
    }

    #[test]
    fn test_handle_key_code() {
        let initial_state = enter_test_dir();
        let initial_state_displayed_paths: Vec<&str> = initial_state
            .displayed_paths
            .iter()
            .map(|fd| fd.shortname.as_str())
            .collect();

        let mut updated = handle_key_code(KeyCode::Char('l'), &mut initial_state.clone()).unwrap();
        assert_eq!(updated.displayed_paths.len(), 1);
        assert_eq!(&updated.displayed_paths[0].shortname, "llkh.py");

        updated = handle_key_code(KeyCode::Backspace, &mut updated).unwrap();
        for file in updated.clone().displayed_paths {
            assert_eq!(
                initial_state_displayed_paths.contains(&file.shortname.as_str()),
                true
            );
        }

        let updated = handle_key_code(KeyCode::Char('d'), &mut updated.clone()).unwrap();
        let updated = handle_key_code(KeyCode::Char('i'), &mut updated.clone()).unwrap();
        let updated = handle_key_code(KeyCode::Char('r'), &mut updated.clone()).unwrap();
        let mut updated = handle_key_code(KeyCode::Char('1'), &mut updated.clone()).unwrap();

        assert_eq!(updated.displayed_paths.len(), 1);
        assert_eq!(updated.displayed_paths[0].shortname, "dir1".to_owned());
        let previous_dir = updated.clone().curr_absolute_path.to_owned();
        let selected = updated.clone().displayed_paths[0].to_owned().absolute;

        let mut updated = handle_key_code(KeyCode::Enter, &mut updated).unwrap();
        assert_eq!(updated.curr_absolute_path, selected);

        let updated = handle_key_code(KeyCode::Left, &mut updated).unwrap();
        assert_eq!(updated.curr_absolute_path, previous_dir);
    }
}
