# Graph App Guidelines

## Scope

These instructions apply to `skala_graph/**`.

## Stack And Structure

- Keep the current separation between:
  - CLI/bootstrap code in `src/main.rs`
  - application state and input handling in `src/app.rs`
  - SQLite access and typed data shaping in `src/data.rs`
  - rendering in `src/ui.rs`
- Prefer small, typed additions over broad refactors.

## Project Facts

- `skala_graph` is a standalone Rust `ratatui` TUI for exploring SKALA SQLite
  databases.
- The intended sample database is
  `../results/qwen-2.5-positive-meltdown.db`.
- Metric selection is checkbox-driven inside the TUI.
- The default visible metrics are:
  - `damage_percent`
  - `energy_production_rate`
  - `advice_new_target_burn_rate`
  - `production_target_rate`
- The graph uses `event.ingame_timestamp` as the x-axis source.
- Multiple selected metrics share one chart by plotting normalised values; raw
  values are still kept for labels and status text.
- The first supported reactor-selection model is:
  - auto-select the only reactor when the database has one
  - allow `--reactor <name>` preselection
  - otherwise let the user switch reactors in the TUI
- “Meaningful metrics” currently means the numeric reactor and target values
  from `snapshot`, plus sparse numeric values from `advice.new_target_burn_rate`
  and `production_target.rate`; IDs, enum codes, generated `pretty_*` columns,
  and text fields are intentionally excluded.
- Sparse or nullable metrics should render as gaps rather than being coerced to
  zero.

## Coding Standards

- Follow the existing Rust 2024 style used in this crate.
- Keep function signatures explicit.
- Use `clap` for CLI parsing rather than hand-rolled argument handling.
- Keep user-facing copy, comments, and documentation in British English.

## Verification

Run checks from `skala_graph/`:

- `cargo fmt`
- `cargo check --offline`
- `cargo clippy --offline -- -D warnings`
- `cargo test --offline`

For documentation-only changes in this directory, a focused review plus
`git diff --check` is sufficient.
