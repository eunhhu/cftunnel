# CFTunnel

A **Ratatui-based TUI** tool for managing Cloudflare Tunnel (`/etc/cloudflared/config.yml`) ingress rules.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- **Ingress Management**: Add, edit, delete hostname → service mappings
- **Protocol Support**: HTTP, HTTPS, SSH, TCP
- **Automatic Backup**: Creates backup before every change
- **Backup Restore**: Restore previous configurations
- **Service Control**: Check status and restart cloudflared service
- **Responsive UI**: Adapts to small terminal sizes
- **Vim-style Navigation**: Use j/k keys to navigate

## Installation

### Build from source

```bash
git clone <repo>
cd cftunnel
cargo build --release
```

### Install to system

```bash
sudo cp target/release/cftunnel /usr/local/bin/
```

## Usage

### TUI Mode

```bash
# Default config path (/etc/cloudflared/config.yml)
sudo cftunnel

# Custom config path
sudo cftunnel -c /path/to/config.yml

# Or use environment variable
CFTUNNEL_CONFIG=/path/to/config.yml sudo -E cftunnel
```

### CLI Mode

```bash
# Show help
cftunnel --help

# Show version
cftunnel --version

# List current rules without TUI
cftunnel --list
```

## CLI Options

| Option | Description |
|--------|-------------|
| `-c, --config <PATH>` | Path to cloudflared config file (default: `/etc/cloudflared/config.yml`) |
| `-l, --list` | List current ingress rules and exit |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Keyboard Shortcuts

### Main Screen

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate menu |
| `Enter` | Select |
| `l` | List mappings |
| `a` | Add new mapping |
| `e` | Edit mapping |
| `d` | Delete mapping |
| `b` | Create backup |
| `r` | Restore backup |
| `s` | Service status |
| `?` | Help |
| `q` | Quit |

### Form Input

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `←`/`→` | Change protocol |
| `Space` | Toggle checkbox |
| `Enter` | Submit |
| `Esc` | Cancel |

## Supported Config Format

```yaml
tunnel: your-tunnel-id
credentials-file: /etc/cloudflared/your-tunnel-id.json

ingress:
  - hostname: api.example.com
    service: http://localhost:3000
  - hostname: ssh.example.com
    service: ssh://127.0.0.1:22
  - hostname: secure.example.com
    service: https://localhost:8443
    originRequest:
      noTLSVerify: true
      httpHostHeader: secure.example.com
  - service: http_status:404  # catch-all (required)
```

## Backups

- Location: `/etc/cloudflared/backups/`
- Format: `config_YYYYMMDD_HHMMSS.yml`
- Current config is also backed up when restoring

## Requirements

- Rust 1.70+
- sudo privileges (for config modification and service restart)
- cloudflared installed and configured

## License

MIT
