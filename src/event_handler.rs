use crate::state_handler::{AppState, KeybindMode};
use crossterm;
use crossterm::event::{KeyCode, KeyModifiers};
use std::error::Error;

pub fn handle_key(key_code: KeyCode, app_state: &mut AppState) -> Result<(), Box<dyn Error>> {
    match app_state.keybind_mode {
        KeybindMode::Normal => handle_normal_mode(key_code, app_state),
        KeybindMode::Insert => handle_key_code_insert(key_code, app_state),
    }
}

fn handle_normal_mode(key_code: KeyCode, app_state: &mut AppState) -> Result<(), Box<dyn Error>> {
    match key_code {
        KeyCode::Char('i') => {
            app_state.keybind_mode = KeybindMode::Insert;
            return Ok(());
        }
        KeyCode::Char('a') => {
            app_state.keybind_mode = KeybindMode::Insert;
            return Ok(());
        }
        KeyCode::Char('h') => {
            app_state.handle_move_back();
            return Ok(());
        }
        KeyCode::Char('j') => {
            app_state.update_selected_index(KeyCode::Down);
            return Ok(());
        }
        KeyCode::Char('k') => {
            app_state.update_selected_index(KeyCode::Up);
            return Ok(());
        }
        KeyCode::Char('l') => {
            app_state.handle_enter();
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
        KeyCode::Char('n') => {
            app_state.toggle_command_mode();
            return Ok(());
        }
        KeyCode::Enter => {
            app_state.handle_enter();
            return Ok(());
        }
        _ => {
            app_state.handle_unsupported_input();
            return Ok(());
        }
    }
}

pub fn handle_key_code_insert(
    key_code: KeyCode,
    app_state: &mut AppState,
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
            if app_state.displayed_paths.len() == 0 {
                app_state.user_input = "".to_owned();
                app_state.displayed_paths = app_state.inner_paths.clone();
            }
            app_state.keybind_mode = KeybindMode::Normal;
            return Ok(());
        }
        _ => return Ok(()),
    }
}

pub fn handle_key_modifier(
    key_code: KeyCode,
    modifier: KeyModifiers,
    app_state: &mut AppState,
) -> Result<(), Box<dyn Error>> {
    if modifier == KeyModifiers::CONTROL {
        match key_code {
            KeyCode::Char('c') => {
                crossterm::terminal::disable_raw_mode()?;
                std::process::exit(0);
            }
            KeyCode::Char('n') => {
                app_state.keybind_mode = KeybindMode::Insert;
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
    use crate::state_handler::AppMode;
    use std::path::Path;

    fn enter_test_dir() -> AppState {
        let test_dir_path = "tests";
        let curr_dir = std::env::current_dir().expect("Could not get current dir");
        let absolute_path = curr_dir
            .join(Path::new(test_dir_path))
            .to_str()
            .expect("Could not turn path to str")
            .to_owned();

        let paths = std::fs::read_dir(&absolute_path).expect("Could not find paths");
        let formatted_paths = file::generate_file_data(paths).expect("Error generating file data");

        return AppState {
            app_mode: AppMode::FileExplorer,
            keybind_mode: KeybindMode::Normal,
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

        handle_key(KeyCode::Char('i'), &mut state).unwrap();
        assert_eq!(state.keybind_mode, KeybindMode::Insert);

        handle_key(KeyCode::Char('l'), &mut state).unwrap();
        assert_eq!(state.displayed_paths.len(), 1);
        assert_eq!(&state.displayed_paths[0].shortname, "llkh.py");

        handle_key(KeyCode::Backspace, &mut state).unwrap();
        for file in state.clone().displayed_paths {
            assert_eq!(
                initial_state_displayed_paths.contains(&file.shortname),
                true
            );
        }

        let new_input = ['d', 'i', 'r', '1'];
        for ch in new_input {
            handle_key(KeyCode::Char(ch), &mut state).unwrap();
        }

        assert_eq!(state.displayed_paths.len(), 1);
        assert_eq!(state.displayed_paths[0].shortname, "dir1".to_owned());
        let previous_dir = state.clone().curr_absolute_path.to_owned();
        let selected = state.clone().displayed_paths[0].to_owned().absolute;

        handle_key(KeyCode::Enter, &mut state).unwrap();
        assert_eq!(state.curr_absolute_path, selected);

        handle_key(KeyCode::Left, &mut state).unwrap();
        assert_eq!(state.curr_absolute_path, previous_dir);

        let new_input = ['t', 'e', 's', 't', 'i', 'n', 'g', '.', 'p', 'y'];
        for ch in new_input {
            handle_key(KeyCode::Char(ch), &mut state).unwrap();
        }

        handle_key(KeyCode::Enter, &mut state).unwrap();
        println!("created file");

        let path_list: Vec<String> = state
            .clone()
            .displayed_paths
            .iter()
            .map(|fd| fd.shortname.to_owned())
            .collect();

        println!("{:?}", path_list);

        assert_eq!(path_list.contains(&"testing.py".to_owned()), true);
        assert_eq!(state.message, "File successfully created".to_owned());
        println!("Enter worked");

        state.selected_index = state
            .displayed_paths
            .iter()
            .position(|fd| fd.shortname.as_str() == "testing.py")
            .unwrap();

        handle_key(KeyCode::Esc, &mut state).unwrap();
        assert_eq!(state.keybind_mode, KeybindMode::Normal);

        handle_key(KeyCode::Char('d'), &mut state).unwrap();
        handle_key(KeyCode::Char('y'), &mut state).unwrap();
        println!("Deleted file");

        let includes_added_file: Vec<&str> = state
            .inner_paths
            .iter()
            .filter(|fd| &fd.shortname == "testing.py")
            .map(|fd| fd.shortname.as_str())
            .collect();
        assert_eq!(includes_added_file.len(), 0);

        handle_key_modifier(KeyCode::Char('n'), KeyModifiers::CONTROL, &mut state).unwrap();
        assert_eq!(state.app_mode, AppMode::Command);

        let new_input = ['e', 'c', 'h', 'o', ' ', 't', 'e', 's', 't'];
        for ch in new_input {
            handle_key_code_insert(KeyCode::Char(ch), &mut state).unwrap();
        }

        handle_key_code_insert(KeyCode::Enter, &mut state).unwrap();
        assert_eq!(state.message, "test".to_owned());

        handle_key_modifier(KeyCode::Char('n'), KeyModifiers::CONTROL, &mut state).unwrap();
        assert_eq!(state.app_mode, AppMode::FileExplorer);
    }
}
