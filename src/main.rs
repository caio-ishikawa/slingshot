use std::error::Error;
use std::panic;
use std::time::Duration;

use crossterm::event::{self, Event, KeyModifiers};
use crossterm::{execute, ExecutableCommand};

mod event_handler;
mod file;
mod state_handler;
mod styles;

fn start_slingshot(starting_state: &state_handler::AppState) -> Result<(), Box<dyn Error>> {
    let polling_interval = Duration::from_millis(10);
    let mut app_state = starting_state.clone();
    //app_state.display_files()?;
    app_state.display()?;

    loop {
        if event::poll(polling_interval)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.modifiers != KeyModifiers::NONE {
                    event_handler::handle_key_modifier(
                        key_event.code,
                        key_event.modifiers,
                        &mut app_state,
                    )?;
                    app_state.display()?;
                } else {
                    event_handler::handle_key(key_event.code, &mut app_state)?;
                    app_state.display()?;
                }
            }
        }
    }
}

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode.");
        println!("Panic occurred: {:?}", panic_info);
    }));

    crossterm::terminal::enable_raw_mode().expect("Could not enable raw mode");
    
    execute!(
        std::io::stdout(),
        crossterm::terminal::DisableLineWrap
    );

    let initial_app_state =
        state_handler::initial_app_state().expect("Error creating initial state");

    start_slingshot(&initial_app_state).unwrap();

    crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode.");
}
