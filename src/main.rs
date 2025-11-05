use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use std::io;

mod storage;
mod service;
mod tui;

use tui::app::App;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode().map_err(|e| anyhow::anyhow!("Failed to enable raw mode: {}. Make sure you're running in a terminal.", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| anyhow::anyhow!("Failed to enter alternate screen: {}. Make sure you're running in a terminal.", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| anyhow::anyhow!("Failed to create terminal: {}. Make sure you're running in a terminal.", e))?;

    // Create app
    let mut app = App::new()?;

    // Main loop
    while !app.should_quit {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code, key.modifiers)?;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
