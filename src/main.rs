mod app;
mod backup;
mod config;
mod event;
mod service;
mod ui;

use anyhow::{Context, Result};
use app::App;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;

const DEFAULT_CONFIG_PATH: &str = "/etc/cloudflared/config.yml";

#[derive(Parser)]
#[command(
    name = "cftunnel",
    author = "eunhhu",
    version,
    about = "Cloudflare Tunnel configuration management TUI",
    long_about = "A TUI tool for managing cloudflared config.yml ingress rules.\n\n\
                  Features:\n\
                  - Add, edit, delete ingress mappings\n\
                  - Automatic backup before changes\n\
                  - Restore from backups\n\
                  - Service status and restart"
)]
struct Cli {
    /// Path to cloudflared config file
    #[arg(short, long, env = "CFTUNNEL_CONFIG", default_value = DEFAULT_CONFIG_PATH)]
    config: PathBuf,

    /// List current ingress rules and exit
    #[arg(short, long)]
    list: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --list: 목록만 출력하고 종료
    if cli.list {
        return print_rules(&cli.config);
    }

    // TUI 실행
    let mut app = App::new(cli.config.clone())
        .with_context(|| format!("Failed to load config: {}", cli.config.display()))?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn print_rules(config_path: &PathBuf) -> Result<()> {
    let config = config::CloudflaredConfig::load(config_path)
        .with_context(|| format!("Failed to load config: {}", config_path.display()))?;

    println!("Tunnel: {}", config.tunnel);
    println!();

    let rules = config.get_ingress_rules();
    if rules.is_empty() {
        println!("No ingress rules configured.");
        return Ok(());
    }

    println!("{:<4} {:<35} {:<30} {}", "#", "Hostname", "Service", "TLS Skip");
    println!("{}", "-".repeat(80));

    for (i, rule) in rules.iter().enumerate() {
        let hostname = rule.hostname.as_deref().unwrap_or("-");
        let tls = rule.origin_request
            .as_ref()
            .and_then(|o| o.no_tls_verify)
            .map(|v| if v { "Yes" } else { "No" })
            .unwrap_or("No");

        println!("{:<4} {:<35} {:<30} {}", i + 1, hostname, rule.service, tls);
    }

    Ok(())
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
