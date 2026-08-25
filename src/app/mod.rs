pub mod state;
pub mod events;

pub use state::AppState;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

use crate::ui::render_app;

pub async fn run_app() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();

    let res = main_loop(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error in Fedora Boost: {:?}", err);
    }

    Ok(())
}

async fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut AppState,
) -> Result<()> {
    let tick_rate = Duration::from_millis(1000);

    while !state.should_quit {
        terminal.draw(|f| render_app(f, state))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                events::handle_key_event(state, key);
            }
        } else {
            state.tick();
        }
    }

    Ok(())
}
