use crate::app::{App, InputField, MainMenuItem, Screen};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub fn poll_event() -> Result<Option<Event>> {
    if event::poll(Duration::from_millis(100))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        // 확인 다이얼로그가 열려있으면 처리
        if app.confirm_action.is_some() {
            return handle_confirm(app, key);
        }

        match app.screen {
            Screen::Main => handle_main(app, key)?,
            Screen::Add => handle_add(app, key)?,
            Screen::Edit => handle_edit(app, key)?,
            Screen::Delete => handle_delete(app, key)?,
            Screen::Backup => handle_backup(app, key)?,
            Screen::Restore => handle_restore(app, key)?,
            Screen::Status => handle_status(app, key)?,
            Screen::Help => handle_help(app, key)?,
        }
    }
    Ok(())
}

fn handle_confirm(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match app.screen {
                Screen::Delete => app.execute_delete()?,
                Screen::Restore => app.execute_restore()?,
                _ => app.confirm_action = None,
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_action = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_main(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.go_to_screen(Screen::Help),

        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),

        KeyCode::Enter => {
            let item = app.selected_menu_item();
            match item {
                MainMenuItem::List => {}
                MainMenuItem::Add => app.go_to_screen(Screen::Add),
                MainMenuItem::Edit => app.go_to_screen(Screen::Edit),
                MainMenuItem::Delete => app.go_to_screen(Screen::Delete),
                MainMenuItem::Backup => {
                    app.go_to_screen(Screen::Backup);
                    app.execute_backup()?;
                }
                MainMenuItem::Restore => app.go_to_screen(Screen::Restore),
                MainMenuItem::Status => app.go_to_screen(Screen::Status),
                MainMenuItem::Help => app.go_to_screen(Screen::Help),
                MainMenuItem::Quit => app.should_quit = true,
            }
        }

        // 단축키
        KeyCode::Char('l') => {}
        KeyCode::Char('a') => app.go_to_screen(Screen::Add),
        KeyCode::Char('e') => app.go_to_screen(Screen::Edit),
        KeyCode::Char('d') => app.go_to_screen(Screen::Delete),
        KeyCode::Char('b') => {
            app.go_to_screen(Screen::Backup);
            app.execute_backup()?;
        }
        KeyCode::Char('r') => app.go_to_screen(Screen::Restore),
        KeyCode::Char('s') => app.go_to_screen(Screen::Status),

        _ => {}
    }
    Ok(())
}

fn handle_add(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.go_back(),
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.input.prev_field();
            } else {
                app.input.next_field();
            }
        }
        KeyCode::BackTab => app.input.prev_field(),
        KeyCode::Enter => app.execute_add()?,
        _ => handle_input_field(app, key),
    }
    Ok(())
}

fn handle_edit(app: &mut App, key: KeyEvent) -> Result<()> {
    // 아직 매핑 선택 중
    if app.input.hostname.is_empty() {
        match key.code {
            KeyCode::Esc => app.go_back(),
            KeyCode::Up | KeyCode::Char('k') => app.list_up(),
            KeyCode::Down | KeyCode::Char('j') => app.list_down(),
            KeyCode::Enter => app.prepare_edit(),
            _ => {}
        }
    } else {
        // 편집 중
        match key.code {
            KeyCode::Esc => app.go_back(),
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    app.input.prev_field();
                } else {
                    app.input.next_field();
                }
            }
            KeyCode::BackTab => app.input.prev_field(),
            KeyCode::Enter => app.execute_edit()?,
            _ => handle_input_field(app, key),
        }
    }
    Ok(())
}

fn handle_delete(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.go_back(),
        KeyCode::Up | KeyCode::Char('k') => app.list_up(),
        KeyCode::Down | KeyCode::Char('j') => app.list_down(),
        KeyCode::Enter => {
            let rules = app.config.get_ingress_rules();
            if let Some(rule) = rules.get(app.list_index) {
                let hostname = rule.hostname.as_deref().unwrap_or("unknown");
                app.confirm_action = Some(format!("delete '{}'", hostname));
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_backup(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.go_back(),
        KeyCode::Enter => app.execute_backup()?,
        _ => {}
    }
    Ok(())
}

fn handle_restore(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.go_back(),
        KeyCode::Up | KeyCode::Char('k') => app.backup_up(),
        KeyCode::Down | KeyCode::Char('j') => app.backup_down(),
        KeyCode::Enter => {
            if !app.backups.is_empty() {
                if let Some(backup) = app.backups.get(app.backup_index) {
                    let name = backup.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("backup");
                    app.confirm_action = Some(format!("restore from '{}'", name));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_status(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.go_back(),
        KeyCode::Char('r') => app.execute_restart()?,
        _ => {}
    }
    Ok(())
}

fn handle_help(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => app.go_back(),
        _ => {}
    }
    Ok(())
}

fn handle_input_field(app: &mut App, key: KeyEvent) {
    app.input.error = None;

    match app.input.active_field {
        InputField::Hostname => handle_text_input(&mut app.input.hostname, key),
        InputField::Protocol => {
            let protocols = crate::app::InputState::protocols();
            match key.code {
                KeyCode::Left => {
                    if app.input.protocol > 0 {
                        app.input.protocol -= 1;
                    }
                }
                KeyCode::Right => {
                    if app.input.protocol < protocols.len() - 1 {
                        app.input.protocol += 1;
                    }
                }
                _ => {}
            }
        }
        InputField::Host => handle_text_input(&mut app.input.host, key),
        InputField::Port => {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    app.input.port.push(c);
                }
                KeyCode::Backspace => {
                    app.input.port.pop();
                }
                _ => {}
            }
        }
        InputField::NoTlsVerify => {
            if key.code == KeyCode::Char(' ') {
                app.input.no_tls_verify = !app.input.no_tls_verify;
            }
        }
    }
}

fn handle_text_input(text: &mut String, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => text.push(c),
        KeyCode::Backspace => { text.pop(); }
        _ => {}
    }
}
