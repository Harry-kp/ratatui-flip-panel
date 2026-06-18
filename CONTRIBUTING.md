# Contributing to ratatui-flip-panel

Thanks for the interest! Bug reports, feature suggestions, and pull requests are all welcome.

## Reporting bugs

Open an issue with reproduction steps. Please include:

- The crate version (e.g. `ratatui-flip-panel = "0.1"`)
- The ratatui version you're using
- Your Rust toolchain (`rustc --version`)
- Operating system

A minimal repro snippet or a link to a small repo helps a lot.

## Proposing changes

For non-trivial changes (new public API, behaviour changes, dep bumps), open an issue first so we can agree on the shape before you write code. For small fixes (typos, doc improvements, obvious clippy fixes), a PR is fine without an issue.

## Development

```shell
git clone https://github.com/Harry-kp/ratatui-flip-panel.git
cd ratatui-flip-panel

cargo test --all-targets        # 8 unit tests + 3 doctests
cargo run --example basic       # interactive demo (press f to flip, q to quit)
```

The repo is a standalone Cargo project — no workspace gymnastics.

## Before opening a PR

These four checks must pass; they're also enforced by CI on every push:

```shell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
```

If you add or modify an example with a visible animation, regenerate its GIF:

```shell
cargo build --release --examples
vhs tapes/<your-example>.tape
```

The recording tapes live under [`tapes/`](./tapes); resulting GIFs land in [`assets/`](./assets) and are referenced from the README.

## Commit style

Conventional commits keep history scannable:

- `feat: add edge_char builder for the midpoint glyph`
- `fix: collapse if-inside-match into match guard`
- `docs(readme): clarify next_tick_in usage`
- `refactor: extract animation curve into pure function`

The body should explain *why*, not *what* the diff already shows.

## Code of conduct

Be kind. Discussion should be technical, focused, and respectful. See [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).

## License

By contributing, you agree your work will be dual-licensed under MIT OR Apache-2.0, matching the project.
