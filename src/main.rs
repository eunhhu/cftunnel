mod app;
mod backup;
mod config;
mod event;
mod service;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;

const DEFAULT_CONFIG_PATH: &str = "/etc/cloudflared/config.yml";

fn main() -> Result<()> {
    let config_path = std::env::var("CFTUNNEL_CONFIG")
        .unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());

    let mut app = App::new(PathBuf::from(&config_path))
        .with_context(|| format!("Failed to load config: {}", config_path))?;

    // Terminal 설정
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 메인 루프
    let result = run(&mut terminal, &mut app);

    // Terminal 복원
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Some(ev) = event::poll_event()? {
            event::handle_event(app, ev)?;
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
