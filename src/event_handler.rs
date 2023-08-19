use crate::state_handler;
use crossterm;
use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;

pub fn handle_key_code(
    key_code: KeyCode,
    app_state: &mut state_handler::AppState,
) -> Result<(), Box<dyn Error>> {
    match key_code {
        KeyCode::Char(c) => {
            app_state.handle_user_input_change(c);
            return Ok(());
        }
        KeyCode::Backspace => {
            app_state.handle_backspace();
            return Ok(());
        }
        KeyCode::Enter => {
            app_state.handle_enter();
            return Ok(());
        }
        KeyCode::Up | KeyCode::Down => {
            app_state.update_selected_index(key_code);
            return Ok(());
        }
        KeyCode::Left => {
            app_state.handle_move_back();
            return Ok(());
        }
        KeyCode::Esc => {
            crossterm::terminal::disable_raw_mode()?;
            std::process::exit(0);
        }
        _ => return Ok(()),
    }
}

pub fn handle_key_modifier(
    // FileExplorer
    key_code: KeyCode,
    modifier: KeyModifiers,
    app_state: &mut state_handler::AppState,
) -> Result<(), Box<dyn Error>> {
    if modifier == KeyModifiers::CONTROL {
        match key_code {
            KeyCode::Char('c') => {
                crossterm::terminal::disable_raw_mode()?;
                std::process::exit(0);
            }
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
            KeyCode::Char('p') => {
                app_state.toggle_command_mode();
                return Ok(());
            }
            _ => {
                app_state.handle_unsupported_input();
                return Ok(());
            }
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
            mode: state_handler::AppMode::FileExplorer,
            curr_absolute_path: absolute_path,
            inner_paths: formatted_paths.clone(),
            displayed_paths: formatted_paths.clone(),
            selected_index: 0,
            user_input: "".to_owned(),
            message: "".to_owned(),
            command_mode: false,
        };
    }

    #[test]
    fn test_state_transitions() {
        let mut state = enter_test_dir();
        let initial_state_displayed_paths: Vec<String> = state
            .clone()
            .displayed_paths
            .iter()
            .map(|fd| fd.shortname.clone())
            .collect();

        handle_key_code(KeyCode::Char('l'), &mut state).unwrap();
        assert_eq!(state.displayed_paths.len(), 1);
        assert_eq!(&state.displayed_paths[0].shortname, "llkh.py");

        handle_key_code(KeyCode::Backspace, &mut state).unwrap();
        for file in state.clone().displayed_paths {
            assert_eq!(
                initial_state_displayed_paths.contains(&file.shortname),
                true
            );
        }

        handle_key_code(KeyCode::Char('d'), &mut state).unwrap();
        handle_key_code(KeyCode::Char('i'), &mut state).unwrap();
        handle_key_code(KeyCode::Char('r'), &mut state).unwrap();
        handle_key_code(KeyCode::Char('1'), &mut state).unwrap();

        assert_eq!(state.displayed_paths.len(), 1);
        assert_eq!(state.displayed_paths[0].shortname, "dir1".to_owned());
        let previous_dir = state.clone().curr_absolute_path.to_owned();
        let selected = state.clone().displayed_paths[0].to_owned().absolute;

        handle_key_code(KeyCode::Enter, &mut state).unwrap();
        assert_eq!(state.curr_absolute_path, selected);

        handle_key_code(KeyCode::Left, &mut state).unwrap();
        assert_eq!(state.curr_absolute_path, previous_dir);

        state.user_input = "testing.py".to_owned();
        handle_key_modifier(KeyCode::Char('n'), KeyModifiers::CONTROL, &mut state).unwrap();

        let path_list: Vec<String> = state
            .clone()
            .displayed_paths
            .iter()
            .map(|fd| fd.shortname.to_owned())
            .collect();

        assert_eq!(path_list.contains(&"testing.py".to_owned()), true);
        assert_eq!(state.message, "File successfully created".to_owned());

        state.selected_index = state
            .displayed_paths
            .iter()
            .position(|fd| fd.shortname.as_str() == "testing.py")
            .unwrap();

        handle_key_modifier(KeyCode::Char('d'), KeyModifiers::CONTROL, &mut state).unwrap();
        handle_key_modifier(KeyCode::Char('y'), KeyModifiers::CONTROL, &mut state).unwrap();

        let includes_added_file: Vec<&str> = state
            .inner_paths
            .iter()
            .filter(|fd| &fd.shortname == "testing.py")
            .map(|fd| fd.shortname.as_str())
            .collect();
        assert_eq!(includes_added_file.len(), 0);
    }
}
