# CFTunnel

Cloudflare Tunnel (`/etc/cloudflared/config.yml`) 설정을 관리하는 **Ratatui 기반 TUI** 도구입니다.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## 기능

- **Ingress 매핑 관리**: 추가, 수정, 삭제
- **프로토콜 지원**: HTTP, HTTPS, SSH, TCP
- **자동 백업**: 변경 전 자동 백업 생성
- **백업 복원**: 이전 설정으로 복원
- **서비스 관리**: cloudflared 서비스 상태 확인 및 재시작
- **Vim 스타일 네비게이션**: j/k로 이동

## 설치

### 빌드

```bash
git clone <repo>
cd cftunnel
cargo build --release
```

### 시스템 설치

```bash
sudo cp target/release/cftunnel /usr/local/bin/
```

## 사용법

```bash
# 기본 경로 (/etc/cloudflared/config.yml)
sudo cftunnel

# 커스텀 경로
CFTUNNEL_CONFIG=/path/to/config.yml sudo -E cftunnel
```

## 키보드 단축키

### 메인 화면

| 키 | 동작 |
|---|---|
| `↑`/`↓` 또는 `j`/`k` | 메뉴 이동 |
| `Enter` | 선택 |
| `l` | 매핑 목록 |
| `a` | 새 매핑 추가 |
| `e` | 매핑 수정 |
| `d` | 매핑 삭제 |
| `b` | 백업 생성 |
| `r` | 백업 복원 |
| `s` | 서비스 상태 |
| `?` | 도움말 |
| `q` | 종료 |

### 폼 입력

| 키 | 동작 |
|---|---|
| `Tab` | 다음 필드 |
| `Shift+Tab` | 이전 필드 |
| `←`/`→` | 프로토콜 변경 |
| `Space` | 체크박스 토글 |
| `Enter` | 제출 |
| `Esc` | 취소 |

## 지원 Config 형식

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
  - service: http_status:404  # catch-all (필수)
```

## 백업

- 위치: `/etc/cloudflared/backups/`
- 형식: `config_YYYYMMDD_HHMMSS.yml`
- 복원 시 현재 설정도 자동 백업

## 라이센스

MIT
