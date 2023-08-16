use crate::state_handler;
use crossterm;
use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;

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
            app_state.handle_enter();
            return Ok(app_state.clone());
        }
        KeyCode::Up | KeyCode::Down => {
            app_state.update_selected_index(key_code);
            return Ok(app_state.clone());
        }
        KeyCode::Left => {
            app_state.handle_move_back();
            return Ok(app_state.clone());
        }
        KeyCode::Esc => {
            crossterm::terminal::disable_raw_mode()?;
            std::process::exit(0);
        }
        _ => return Ok(app_state.clone()),
    }
}

pub fn handle_key_modifier(
    key_code: KeyCode,
    modifier: KeyModifiers,
    app_state: &mut state_handler::AppState,
) -> Result<(), Box<dyn Error>> {
    if modifier == KeyModifiers::CONTROL {
        match key_code {
            KeyCode::Char('n') => {
                app_state.handle_create();
                return Ok(());
            }
            KeyCode::Char('k') => {
                app_state.update_selected_index(KeyCode::Up);
                return Ok(());
            }
            KeyCode::Char('j') => {
                app_state.update_selected_index(KeyCode::Down);
                return Ok(());
            }
            KeyCode::Char('d') => {
                app_state.handle_mark_delete();
                return Ok(());
            }
            KeyCode::Char('y') => {
                app_state.handle_confirm_delete();
                return Ok(());
            }
            _ => panic!("not implemented"),
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
            message: "".to_owned(),
        };
    }

    #[test]
    fn test_state_transitions() {
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

        let mut updated = handle_key_code(KeyCode::Left, &mut updated).unwrap();
        assert_eq!(updated.curr_absolute_path, previous_dir);

        updated.search_term = "testing.py".to_owned();
        handle_key_modifier(KeyCode::Char('n'), KeyModifiers::CONTROL, &mut updated).unwrap();

        let path_list: Vec<String> = updated
            .clone()
            .displayed_paths
            .iter()
            .map(|fd| fd.shortname.to_owned())
            .collect();

        assert_eq!(path_list.contains(&"testing.py".to_owned()), true);
        assert_eq!(updated.message, "File successfully created".to_owned());

        updated.selected_index = updated
            .displayed_paths
            .iter()
            .position(|fd| fd.shortname.as_str() == "testing.py")
            .unwrap();

        handle_key_modifier(KeyCode::Char('d'), KeyModifiers::CONTROL, &mut updated).unwrap();
        handle_key_modifier(KeyCode::Char('y'), KeyModifiers::CONTROL, &mut updated).unwrap();

        let includes_added_file: Vec<&str> = updated
            .inner_paths
            .iter()
            .filter(|fd| &fd.shortname == "testing.py")
            .map(|fd| fd.shortname.as_str())
            .collect();
        assert_eq!(includes_added_file.len(), 0);
    }
}
