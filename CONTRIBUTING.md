# Contributing to bridge-echo

## Development

```sh
cargo build
cargo test
cargo clippy
cargo fmt
```

All four must pass before submitting a PR.

## Pull Requests

1. Fork the repo and create a feature branch
2. Make your changes
3. Run `cargo fmt && cargo clippy && cargo test`
4. Open a PR against `main`

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- No clippy warnings allowed
- Write tests for new functionality
