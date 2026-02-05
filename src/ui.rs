use crate::app::{App, InputField, InputState, MainMenuItem, Screen};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Wrap},
    Frame,
};

const LOGO_LARGE: &str = r#"   _____ _____ _____                      _
  / ____|  ___|_   _|   _ _ __  _ __   ___| |
 | |    | |_    | || | | | '_ \| '_ \ / _ \ |
 | |___ |  _|   | || |_| | | | | | | |  __/ |
  \____|_|     |_| \__,_|_| |_|_| |_|\___|_|"#;

const LOGO_SMALL: &str = "[ CFTunnel ]";

const MIN_WIDTH: u16 = 40;
const MIN_HEIGHT: u16 = 15;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // 최소 크기 체크
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_size_warning(frame, area);
        return;
    }

    let is_compact = area.height < 25 || area.width < 80;

    // 메인 레이아웃 - 화면 크기에 따라 조정
    let chunks = if is_compact {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // 로고 (한 줄)
                Constraint::Min(8),     // 컨텐츠
                Constraint::Length(1),  // 상태바
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // 로고
                Constraint::Min(10),    // 컨텐츠
                Constraint::Length(3),  // 상태바
            ])
            .split(area)
    };

    render_header(frame, chunks[0], is_compact);
    render_content(frame, app, chunks[1], is_compact);
    render_statusbar(frame, app, chunks[2], is_compact);

    // 확인 다이얼로그
    if app.confirm_action.is_some() {
        render_confirm_dialog(frame, app, area);
    }
}

fn render_size_warning(frame: &mut Frame, area: Rect) {
    let text = format!(
        "Terminal too small!\nMin: {}x{}\nCurrent: {}x{}",
        MIN_WIDTH, MIN_HEIGHT, area.width, area.height
    );
    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::Red).bold())
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_header(frame: &mut Frame, area: Rect, is_compact: bool) {
    if is_compact {
        let logo = Paragraph::new(LOGO_SMALL)
            .style(Style::default().fg(Color::Cyan).bold());
        frame.render_widget(logo, area);
    } else {
        let logo = Paragraph::new(LOGO_LARGE)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(logo, area);
    }
}

fn render_content(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    match app.screen {
        Screen::Main => render_main_screen(frame, app, area, is_compact),
        Screen::Add => render_add_screen(frame, app, area, is_compact),
        Screen::Edit => render_edit_screen(frame, app, area, is_compact),
        Screen::Delete => render_delete_screen(frame, app, area),
        Screen::Backup => render_backup_screen(frame, app, area),
        Screen::Restore => render_restore_screen(frame, app, area),
        Screen::Status => render_status_screen(frame, app, area, is_compact),
        Screen::Help => render_help_screen(frame, area, is_compact),
    }
}

fn render_main_screen(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    let menu_width = if is_compact { 20 } else { 25 };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(menu_width), Constraint::Min(20)])
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

            let content = if is_compact {
                format!("[{}]{}", item.shortcut(), item.label())
            } else {
                format!(" [{}] {}", item.shortcut(), item.label())
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu "));
    frame.render_widget(menu, chunks[0]);

    // 매핑 목록
    render_mappings_table(frame, app, chunks[1], is_compact);
}

fn render_mappings_table(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    let rules = app.config.get_ingress_rules();

    if is_compact {
        // 컴팩트 모드: 간단한 리스트
        let items: Vec<ListItem> = rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let hostname = rule.hostname.as_deref().unwrap_or("-");
                let port = rule.get_port().map(|p| p.to_string()).unwrap_or_default();
                let style = if i == app.list_index && matches!(app.screen, Screen::Edit | Screen::Delete) {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{} :{}", hostname, port)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Rules "));
        frame.render_widget(list, area);
    } else {
        // 일반 모드: 테이블
        let header = Row::new(vec!["#", "Hostname", "Service", "TLS"])
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
                    .map(|v| if v { "Y" } else { "N" })
                    .unwrap_or("N");

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
            Constraint::Percentage(35),
            Constraint::Percentage(55),
            Constraint::Length(4),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(" Ingress Rules "));

        frame.render_widget(table, area);
    }
}

fn render_add_screen(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Add New Mapping ")
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    render_input_form(frame, app, inner, is_compact);
}

fn render_edit_screen(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    if app.input.hostname.is_empty() {
        // 매핑 선택 모드
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Select to Edit (↑↓ Enter) ")
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);
        render_selectable_list(frame, app, inner);
    } else {
        // 편집 폼
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Edit: {} ", app.input.hostname))
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);
        render_input_form(frame, app, inner, is_compact);
    }
}

fn render_delete_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delete (↑↓ Enter) ")
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

fn render_input_form(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    let active = app.input.active_field;

    if is_compact {
        // 컴팩트 모드: 인라인 표시
        render_compact_form(frame, app, area, active);
    } else {
        // 일반 모드: 각 필드 별도 박스
        render_full_form(frame, app, area, active);
    }
}

fn render_compact_form(frame: &mut Frame, app: &App, area: Rect, active: InputField) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // hostname
            Constraint::Length(1), // protocol + host
            Constraint::Length(1), // port + tls
            Constraint::Length(1), // error/help
            Constraint::Min(0),
        ])
        .split(area);

    // Hostname
    let hn_style = if active == InputField::Hostname { Style::default().fg(Color::Cyan).bold() } else { Style::default() };
    let hostname = Line::from(vec![
        Span::styled("Host: ", Style::default().fg(Color::Yellow)),
        Span::styled(&app.input.hostname, hn_style),
        if active == InputField::Hostname { Span::raw("_") } else { Span::raw("") },
    ]);
    frame.render_widget(Paragraph::new(hostname), chunks[0]);

    // Protocol + Host
    let proto_style = if active == InputField::Protocol { Style::default().fg(Color::Cyan).bold() } else { Style::default() };
    let host_style = if active == InputField::Host { Style::default().fg(Color::Cyan).bold() } else { Style::default() };
    let protocols = InputState::protocols();
    let proto_host = Line::from(vec![
        Span::styled("Proto: ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("<{}>", protocols[app.input.protocol]), proto_style),
        Span::raw("  "),
        Span::styled("To: ", Style::default().fg(Color::Yellow)),
        Span::styled(&app.input.host, host_style),
        if active == InputField::Host { Span::raw("_") } else { Span::raw("") },
    ]);
    frame.render_widget(Paragraph::new(proto_host), chunks[1]);

    // Port + TLS
    let port_style = if active == InputField::Port { Style::default().fg(Color::Cyan).bold() } else { Style::default() };
    let tls_style = if active == InputField::NoTlsVerify { Style::default().fg(Color::Cyan).bold() } else { Style::default() };
    let tls_check = if app.input.no_tls_verify { "[x]" } else { "[ ]" };
    let port_tls = Line::from(vec![
        Span::styled("Port: ", Style::default().fg(Color::Yellow)),
        Span::styled(&app.input.port, port_style),
        if active == InputField::Port { Span::raw("_") } else { Span::raw("") },
        Span::raw("  "),
        Span::styled(format!("{} SkipTLS", tls_check), tls_style),
    ]);
    frame.render_widget(Paragraph::new(port_tls), chunks[2]);

    // Error or help
    let bottom = if let Some(ref err) = app.input.error {
        Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red))
    } else {
        Paragraph::new("Tab:Next Enter:Submit Esc:Cancel").style(Style::default().fg(Color::DarkGray))
    };
    frame.render_widget(bottom, chunks[3]);
}

fn render_full_form(frame: &mut Frame, app: &App, area: Rect, active: InputField) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // hostname
            Constraint::Length(3), // protocol
            Constraint::Length(3), // host
            Constraint::Length(3), // port
            Constraint::Length(3), // noTlsVerify
            Constraint::Min(1),    // error + help
        ])
        .split(area);

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
        .block(Block::default().borders(Borders::ALL).title(" TLS (Space) ").border_style(tls_style));
    frame.render_widget(tls, chunks[4]);

    // Error + Help
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(chunks[5]);

    if let Some(ref err) = app.input.error {
        let error = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red).bold());
        frame.render_widget(error, bottom_chunks[0]);
    }

    let help = Paragraph::new("Tab: Next | Shift+Tab: Prev | Enter: Submit | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, bottom_chunks[1]);
}

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
        Paragraph::new(msg.as_str()).style(style).wrap(Wrap { trim: true })
    } else {
        Paragraph::new("Press Enter to create a backup\nPress Esc to go back")
    };

    frame.render_widget(text, inner);
}

fn render_restore_screen(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Restore Backup ")
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

fn render_status_screen(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Service Status ")
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some((active, ref status)) = app.service_status {
        let status_indicator = if active {
            Span::styled("● ACTIVE", Style::default().fg(Color::Green).bold())
        } else {
            Span::styled("● INACTIVE", Style::default().fg(Color::Red).bold())
        };

        if is_compact {
            let header = Paragraph::new(Line::from(vec![
                Span::raw("cloudflared: "),
                status_indicator,
                Span::raw(" [r]restart"),
            ]));
            frame.render_widget(header, inner);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(3)])
                .split(inner);

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
        }
    } else {
        let text = Paragraph::new("Loading...");
        frame.render_widget(text, inner);
    }
}

fn render_help_screen(frame: &mut Frame, area: Rect, is_compact: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = if is_compact {
        vec![
            "Navigation: ↑↓/jk Enter Esc",
            "l:List a:Add e:Edit d:Delete",
            "b:Backup r:Restore s:Status",
            "?:Help q:Quit",
            "Form: Tab ←→ Space",
        ]
    } else {
        vec![
            "",
            "  Navigation:",
            "    ↑/↓, j/k    Move selection",
            "    Enter       Confirm / Select",
            "    Esc         Go back / Cancel",
            "",
            "  Shortcuts:",
            "    l  List     a  Add      e  Edit",
            "    d  Delete   b  Backup   r  Restore",
            "    s  Status   ?  Help     q  Quit",
            "",
            "  In Forms:",
            "    Tab/Shift+Tab  Navigate fields",
            "    ←/→            Change protocol",
            "    Space          Toggle checkbox",
        ]
    };

    let text = Text::from(
        help_text.iter()
            .map(|&s| Line::from(s))
            .collect::<Vec<_>>()
    );

    let para = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn render_statusbar(frame: &mut Frame, app: &App, area: Rect, is_compact: bool) {
    if is_compact {
        // 컴팩트: 한 줄
        let content = if let Some((ref msg, is_err)) = app.message {
            let style = if is_err { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Green) };
            Paragraph::new(msg.as_str()).style(style)
        } else {
            let hint = match app.screen {
                Screen::Main => "↑↓:Nav Enter:Sel q:Quit",
                Screen::Add | Screen::Edit => "Tab:Next Enter:OK Esc:Back",
                _ => "Esc:Back",
            };
            Paragraph::new(hint).style(Style::default().fg(Color::DarkGray))
        };
        frame.render_widget(content, area);
    } else {
        // 일반: 두 섹션
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

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

        let config_path = app.config_path.display().to_string();
        let path_widget = Paragraph::new(config_path)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Config "));
        frame.render_widget(path_widget, chunks[1]);
    }
}

fn render_confirm_dialog(frame: &mut Frame, app: &App, area: Rect) {
    // 다이얼로그 크기를 화면에 맞게 조정
    let width = (area.width.saturating_sub(4)).min(50);
    let height = (area.height.saturating_sub(4)).min(7);

    let dialog_area = centered_rect_fixed(width, height, area);

    frame.render_widget(Clear, dialog_area);

    let action = app.confirm_action.as_deref().unwrap_or("this action");
    let text = format!("{}?\n\n[Y] Yes  [N] No", action);

    let dialog = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm ")
                .border_style(Style::default().fg(Color::Yellow))
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(dialog, dialog_area);
}

fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}
