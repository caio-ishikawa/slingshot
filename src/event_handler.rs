use crate::state_handler;
use crossterm::event::KeyCode;
use std::borrow::Cow;
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
        _ => return Ok(app_state.clone())
    }
}
