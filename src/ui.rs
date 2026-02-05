use crate::app::{App, InputField, MainMenuItem, Screen};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

const LOGO: &str = r#"
   _____ _____ _____                      _
  / ____|  ___|_   _|   _ _ __  _ __   ___| |
 | |    | |_    | || | | | '_ \| '_ \ / _ \ |
 | |___ |  _|   | || |_| | | | | | | |  __/ |
  \____|_|     |_| \__,_|_| |_|_| |_|\___|_|
"#;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // 메인 레이아웃
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // 로고
            Constraint::Min(10),    // 컨텐츠
            Constraint::Length(3),  // 상태바
        ])
        .split(area);

    render_header(frame, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_statusbar(frame, app, chunks[2]);

    // 확인 다이얼로그
    if app.confirm_action.is_some() {
        render_confirm_dialog(frame, app, area);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let logo = Paragraph::new(LOGO)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default());
    frame.render_widget(logo, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.screen {
        Screen::Main => render_main_screen(frame, app, area),
        Screen::Add => render_add_screen(frame, app, area),
        Screen::Edit => render_edit_screen(frame, app, area),
        Screen::Delete => render_delete_screen(frame, app, area),
        Screen::Backup => render_backup_screen(frame, app, area),
        Screen::Restore => render_restore_screen(frame, app, area),
        Screen::Status => render_status_screen(frame, app, area),
        Screen::Help => render_help_screen(frame, area),
    }
}

fn render_main_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(25), Constraint::Min(50)])
        .split(area);

    // 메뉴
    let items: Vec<ListItem> = MainMenuItem::all()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.menu_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };

            let content = format!(" [{}] {}", item.shortcut(), item.label());
            ListItem::new(content).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu "));
    frame.render_widget(menu, chunks[0]);

    // 매핑 목록
    render_mappings_table(frame, app, chunks[1]);
}

fn render_mappings_table(frame: &mut Frame, app: &App, area: Rect) {
    let rules = app.config.get_ingress_rules();

    let header = Row::new(vec!["#", "Hostname", "Service", "TLS Skip"])
        .style(Style::default().fg(Color::Yellow).bold())
        .height(1);

    let rows: Vec<Row> = rules
        .iter()
        .enumerate()
        .map(|(i, rule)| {
            let hostname = rule.hostname.as_deref().unwrap_or("-");
            let tls = rule.origin_request
                .as_ref()
                .and_then(|o| o.no_tls_verify)
                .map(|v| if v { "Yes" } else { "No" })
                .unwrap_or("No");

            let style = if i == app.list_index && matches!(app.screen, Screen::Edit | Screen::Delete) {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };

            Row::new(vec![
                (i + 1).to_string(),
                hostname.to_string(),
                rule.service.clone(),
                tls.to_string(),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(35),
        Constraint::Min(25),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Ingress Rules "));

    frame.render_widget(table, area);
}

fn render_add_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Add New Mapping ")
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    render_input_form(frame, app, inner);
}

fn render_edit_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // 선택할 매핑 목록
    let select_block = Block::default()
        .borders(Borders::ALL)
        .title(" Select Mapping to Edit (↑↓ to select, Enter to confirm) ");
    let select_inner = select_block.inner(chunks[0]);
    frame.render_widget(select_block, chunks[0]);
    render_selectable_list(frame, app, select_inner);

    // 편집 폼
    let edit_block = Block::default()
        .borders(Borders::ALL)
        .title(" Edit ")
        .border_style(Style::default().fg(Color::Yellow));
    let edit_inner = edit_block.inner(chunks[1]);
    frame.render_widget(edit_block, chunks[1]);

    if !app.input.hostname.is_empty() {
        render_input_form(frame, app, edit_inner);
    }
}

fn render_delete_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select Mapping to Delete (↑↓ to select, Enter to delete) ")
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_selectable_list(frame, app, inner);
}

fn render_selectable_list(frame: &mut Frame, app: &App, area: Rect) {
    let rules = app.config.get_ingress_rules();

    let items: Vec<ListItem> = rules
        .iter()
        .enumerate()
        .map(|(i, rule)| {
            let hostname = rule.hostname.as_deref().unwrap_or("-");
            let style = if i == app.list_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default()
            };

            let content = format!(" {} → {}", hostname, rule.service);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);
}

fn render_input_form(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // hostname
            Constraint::Length(3), // protocol
            Constraint::Length(3), // host
            Constraint::Length(3), // port
            Constraint::Length(3), // noTlsVerify
            Constraint::Length(2), // error
            Constraint::Min(1),    // help
        ])
        .margin(1)
        .split(area);

    let active = app.input.active_field;

    // Hostname
    let hostname_style = field_style(active == InputField::Hostname);
    let hostname = Paragraph::new(app.input.hostname.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Hostname ").border_style(hostname_style));
    frame.render_widget(hostname, chunks[0]);

    // Protocol
    let proto_style = field_style(active == InputField::Protocol);
    let protocols = InputState::protocols();
    let proto_text = format!("< {} >", protocols[app.input.protocol]);
    let protocol = Paragraph::new(proto_text)
        .block(Block::default().borders(Borders::ALL).title(" Protocol (←→) ").border_style(proto_style));
    frame.render_widget(protocol, chunks[1]);

    // Host
    let host_style = field_style(active == InputField::Host);
    let host = Paragraph::new(app.input.host.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Host ").border_style(host_style));
    frame.render_widget(host, chunks[2]);

    // Port
    let port_style = field_style(active == InputField::Port);
    let port = Paragraph::new(app.input.port.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Port ").border_style(port_style));
    frame.render_widget(port, chunks[3]);

    // NoTlsVerify
    let tls_style = field_style(active == InputField::NoTlsVerify);
    let tls_text = if app.input.no_tls_verify { "[x] Skip TLS Verify" } else { "[ ] Skip TLS Verify" };
    let tls = Paragraph::new(tls_text)
        .block(Block::default().borders(Borders::ALL).title(" TLS (Space to toggle) ").border_style(tls_style));
    frame.render_widget(tls, chunks[4]);

    // Error
    if let Some(ref err) = app.input.error {
        let error = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red).bold());
        frame.render_widget(error, chunks[5]);
    }

    // Help
    let help = Paragraph::new("Tab: Next field | Shift+Tab: Prev | Enter: Submit | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[6]);
}

use crate::app::InputState;

fn field_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

fn render_backup_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Backup ")
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = if let Some((ref msg, is_err)) = app.message {
        let style = if is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        Paragraph::new(msg.as_str()).style(style)
    } else {
        Paragraph::new("Press Enter to create a backup\nPress Esc to go back")
    };

    frame.render_widget(text, inner);
}

fn render_restore_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select Backup to Restore ")
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.backups.is_empty() {
        let text = Paragraph::new("No backups found");
        frame.render_widget(text, inner);
        return;
    }

    let items: Vec<ListItem> = app.backups
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let style = if i == app.backup_index {
                Style::default().fg(Color::Black).bg(Color::Magenta).bold()
            } else {
                Style::default()
            };

            ListItem::new(format!(" {}", name)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_status_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Service Status ")
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .margin(1)
        .split(inner);

    if let Some((active, ref status)) = app.service_status {
        let status_indicator = if active {
            Span::styled("● ACTIVE", Style::default().fg(Color::Green).bold())
        } else {
            Span::styled("● INACTIVE", Style::default().fg(Color::Red).bold())
        };

        let header = Paragraph::new(Line::from(vec![
            Span::raw("cloudflared: "),
            status_indicator,
            Span::raw("  |  Press 'r' to restart"),
        ]));
        frame.render_widget(header, chunks[0]);

        let detail = Paragraph::new(status.as_str())
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(detail, chunks[1]);
    } else {
        let text = Paragraph::new("Loading...");
        frame.render_widget(text, inner);
    }
}

fn render_help_screen(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        "",
        "  Navigation:",
        "    ↑/↓, j/k    Move selection",
        "    Enter       Confirm / Select",
        "    Esc         Go back / Cancel",
        "",
        "  Shortcuts:",
        "    l           List mappings",
        "    a           Add new mapping",
        "    e           Edit mapping",
        "    d           Delete mapping",
        "    b           Create backup",
        "    r           Restore backup / Restart service",
        "    s           Service status",
        "    ?           Show this help",
        "    q           Quit",
        "",
        "  In Forms:",
        "    Tab         Next field",
        "    Shift+Tab   Previous field",
        "    ←/→         Change protocol",
        "    Space       Toggle checkbox",
        "",
    ];

    let text = Text::from(
        help_text.iter()
            .map(|&s| Line::from(s))
            .collect::<Vec<_>>()
    );

    let para = Paragraph::new(text);
    frame.render_widget(para, inner);
}

fn render_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // 메시지
    let msg_widget = if let Some((ref msg, is_err)) = app.message {
        let style = if is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        Paragraph::new(msg.as_str()).style(style)
    } else {
        let hint = match app.screen {
            Screen::Main => "↑↓/jk: Navigate | Enter: Select | q: Quit",
            Screen::Add | Screen::Edit => "Tab: Next | Enter: Submit | Esc: Cancel",
            Screen::Delete | Screen::Restore => "↑↓: Select | Enter: Confirm | Esc: Cancel",
            Screen::Status => "r: Restart | Esc: Back",
            _ => "Esc: Back",
        };
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray))
    };

    let msg_block = Block::default().borders(Borders::ALL);
    frame.render_widget(msg_widget.block(msg_block), chunks[0]);

    // Config 경로
    let config_path = app.config_path.display().to_string();
    let path_widget = Paragraph::new(config_path)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Config "));
    frame.render_widget(path_widget, chunks[1]);
}

fn render_confirm_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog_area = centered_rect(50, 20, area);

    frame.render_widget(Clear, dialog_area);

    let action = app.confirm_action.as_deref().unwrap_or("this action");
    let text = format!(
        "\n  Are you sure you want to {}?\n\n  [Y] Yes    [N] No",
        action
    );

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm ")
                .border_style(Style::default().fg(Color::Yellow))
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(dialog, dialog_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
