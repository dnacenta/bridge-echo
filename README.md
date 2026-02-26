# bridge-echo

HTTP bridge for Claude Code CLI. Receives HTTP POST requests with a message, runs `claude -p`, returns the response.

Built for [voice-echo](https://github.com/dnacenta/voice-echo) — lets n8n and other services talk to Claude Code over HTTP.

## Features

- **Session continuity** — per-channel conversations via Claude's `-r` flag
- **Trust levels** — channels mapped to trusted/verified/untrusted with security context injection
- **Injection detection** — 26 regex patterns scanned on non-trusted input
- **Persona injection** — optional SELF.md file passed via `--append-system-prompt`
- **Zero config files** — all configuration through environment variables

## Install

### From source

```sh
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/dnacenta/bridge-echo/releases).

## Usage

```sh
bridge-echo
```

The server starts on `0.0.0.0:3100` by default.

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `BRIDGE_ECHO_HOST` | `0.0.0.0` | Listen address |
| `BRIDGE_ECHO_PORT` | `3100` | Listen port |
| `BRIDGE_ECHO_TIMEOUT` | `600` | Claude subprocess timeout (seconds) |
| `BRIDGE_ECHO_SESSION_TTL` | `3600` | Session expiry (seconds) |
| `BRIDGE_ECHO_CLAUDE_BIN` | `claude` | Path to Claude CLI binary |
| `BRIDGE_ECHO_SELF_PATH` | — | Path to persona file (SELF.md) |
| `BRIDGE_ECHO_HOME` | `$HOME` | Working directory for Claude |
| `RUST_LOG` | `bridge_echo=info` | Log level filter |

## API

### POST /chat

```sh
curl -X POST http://localhost:3100/chat \
  -H 'Content-Type: application/json' \
  -d '{"message": "hello", "channel": "slack"}'
```

Response:
```json
{"response": "Hello! How can I help?"}
```

`channel` defaults to `"slack"` if omitted.

### GET /health

```json
{"status": "ok"}
```

## Trust Levels

Channels are mapped to trust levels that control the security context injected into prompts:

- **Trusted** (`reflection`, `system`) — no restrictions, self-initiated
- **Verified** (`slack`, `slack-echo`, `discord`, `discord-echo`) — authenticated but treated as user input
- **Untrusted** (everything else) — full lockdown, conversation only

## Systemd

A service file is provided in `service/bridge-echo.service`. Copy it to `/etc/systemd/system/` and adjust paths as needed.

## License

MIT
