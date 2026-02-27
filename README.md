# bridge-echo

[![License: MIT](https://img.shields.io/github/license/dnacenta/bridge-echo)](LICENSE)
[![Version](https://img.shields.io/github/v/tag/dnacenta/bridge-echo?label=version&color=green)](https://github.com/dnacenta/bridge-echo/tags)

HTTP bridge for Claude Code CLI. Lets external services talk to Claude Code over HTTP — send a message, get a response.

## Why

Claude Code is powerful but locked to the terminal. If you want other services to interact with it — workflow automation, chatbots, webhooks, scheduled tasks — there's no built-in HTTP interface.

bridge-echo solves this with a single-purpose server: receive a POST with a message, run `claude -p`, return the response. On top of that:

- **Session continuity.** Per-channel conversations via Claude's `-r` flag. Messages on the same channel resume the same session automatically.
- **Trust-aware security.** Channels are mapped to trust levels (trusted, verified, untrusted). Each level injects appropriate security context into the prompt, so Claude knows how much to trust the input.
- **Injection detection.** 26 regex patterns scanned on non-trusted input. Suspicious messages get a security warning prepended to the prompt.
- **Persona injection.** Optional system prompt file passed via `--append-system-prompt`, so Claude maintains a consistent persona across all channels.
- **Zero config files.** Everything through environment variables. No YAML, no TOML, no JSON config.

## How It Works

```
  External Service           bridge-echo              Claude Code CLI
  (n8n, bot, webhook)        (HTTP server)            (subprocess)
  ─────────────────          ─────────────            ──────────────

  POST /chat ──────────────▶ Resolve trust level
  {"message": "...",         for channel
   "channel": "mychat"}       │
                             ├──▶ Scan for injection patterns
                             │    (non-trusted channels only)
                             │
                             ├──▶ Build prompt with security context
                             │    + optional injection warning
                             │
                             ├──▶ Look up session for channel
                             │
                             ├──▶ Read persona file (if configured)
                             │
                             ├──▶ Spawn: claude -p "<prompt>"
                             │         --output-format json
                             │         --dangerously-skip-permissions
                             │         [-r <session_id>]
                             │         [--append-system-prompt <persona>]
                             │
                             ◀──── Parse JSON output ◀───────────────
                             │
                             ├──▶ Update session store
                             │
  ◀──────────────────────────┘
  {"response": "..."}
```

### Trust Levels

Every channel is mapped to a trust level that controls the security context injected into the prompt:

```
  Trust Level       Behavior
  ───────────       ────────
  Trusted           No restrictions — self-initiated, internal use
  Verified          Security boundaries applied — authenticated
                    but treated as user input
  Untrusted         Full lockdown — conversation only, no tool use
```

Channel-to-trust mappings live in `src/trust.rs`. Edit them to match your setup — map your internal channels to Trusted, authenticated user-facing channels to Verified, and leave everything else as Untrusted.

### Injection Detection

26 case-insensitive regex patterns compiled into a `RegexSet` at startup. Covers instruction override, persona hijack, permission bypass, prompt extraction, dangerous commands, and jailbreak attempts. When a match is found on a non-trusted channel, a security warning is prepended to the prompt.

## Installation

### cargo install (recommended)

```bash
cargo install --path .
```

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/dnacenta/bridge-echo/releases/latest) for:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin` (Apple Silicon)

Extract and run:

```bash
tar xzf bridge-echo-<target>.tar.gz
./bridge-echo
```

### From source

```bash
git clone https://github.com/dnacenta/bridge-echo.git
cd bridge-echo
cargo build --release
./target/release/bridge-echo
```

## Configuration

All configuration through environment variables. No config files.

| Variable | Default | Description |
|---|---|---|
| `BRIDGE_ECHO_HOST` | `0.0.0.0` | Listen address |
| `BRIDGE_ECHO_PORT` | `3100` | Listen port |
| `BRIDGE_ECHO_TIMEOUT` | `600` | Claude subprocess timeout (seconds) |
| `BRIDGE_ECHO_SESSION_TTL` | `3600` | Session expiry (seconds) |
| `BRIDGE_ECHO_CLAUDE_BIN` | `claude` | Path to Claude CLI binary |
| `BRIDGE_ECHO_SELF_PATH` | — | Path to persona/system prompt file |
| `BRIDGE_ECHO_HOME` | `$HOME` | Working directory for Claude |
| `RUST_LOG` | `bridge_echo=info` | Log level filter |

## API

### POST /chat

```bash
curl -X POST http://localhost:3100/chat \
  -H 'Content-Type: application/json' \
  -d '{"message": "hello", "channel": "mychat"}'
```

```json
{"response": "Hello! How can I help?"}
```

| Field | Required | Default | Description |
|---|---|---|---|
| `message` | yes | — | The message to send to Claude |
| `channel` | no | `"default"` | Channel name (determines trust level and session) |

Responses always return 200 with the response text — including errors and timeouts. Only 400 for malformed input (invalid JSON, missing message).

### GET /health

```json
{"status": "ok"}
```

## Running as a Service

A template systemd unit is provided in `service/bridge-echo.service`. Copy it to `/etc/systemd/system/`, fill in the placeholders, then:

```bash
systemctl daemon-reload
systemctl enable --now bridge-echo
```

## Project Structure

```
bridge-echo/
  src/
    main.rs ··················· Entry point, tracing, server bootstrap
    config.rs ················· Env-var config with defaults
    state.rs ·················· AppState: config + sessions + detector
    router.rs ················· Axum router (/chat + /health)
    claude.rs ················· Async subprocess invocation
    session.rs ················ In-memory session store with TTL
    trust.rs ·················· Channel → trust mapping, context strings
    injection.rs ·············· 26 regex patterns, RegexSet
    prompt.rs ················· Assemble final prompt with context
    handlers/
      chat.rs ················· POST /chat handler
      health.rs ··············· GET /health handler
  service/
    bridge-echo.service ······· Systemd unit template
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
