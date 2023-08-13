mod event_handler;
mod file;
mod state_handler;
mod styles;

use crossterm::event;
use std::error::Error;
use std::panic;

fn start_slingshot(starting_state: &state_handler::AppState) -> Result<(), Box<dyn Error>> {
    let mut app_state = starting_state.clone();
    app_state.display()?;

    loop {
        if let event::Event::Key(key_event) = event::read()? {
            app_state =
                event_handler::handle_key_code(key_event.code, &mut app_state)?;
            app_state.display()?;
        }
    }
}

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode.");
        println!("Panic occurred: {:?}", panic_info);
    }));

    crossterm::terminal::enable_raw_mode().expect("Could not enable raw mode");

    let initial_app_state =
        state_handler::initial_app_state().expect("Error creating initial state");

    start_slingshot(&initial_app_state).unwrap();

    crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode.");
}
