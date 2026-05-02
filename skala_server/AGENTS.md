# Server Guidelines

## Scope

These instructions apply to `skala_server/**`, except vendored dependencies
under `skala_server/vendor/**`.

## Coding Standards

- Follow the existing Rust 2024 workspace layout, module boundaries, and Axum
  route patterns.
- Keep Rust function signatures explicit and follow the existing derive, serde,
  SQLx, and quicktype conventions.
- When database schema or query shapes change, keep migrations and tracked
  `.sqlx/` query metadata in sync.
- Snapshot files under Rust test `snapshots/` are fixtures. Update them only
  when the behaviour change is intentional.

## Client Contract

- Server request and response types that derive `quicktype::Quicktype` feed the
  client type contract.
- When those types change, run `scripts/check_quicktype_specs` from
  `skala_server/` and update `../skala_client/skala/server_types.yue` if the
  script reports a mismatch.

## Verification

Run server checks from `skala_server/`:

- `cargo fmt -- --check`
- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo clippy --tests -- -D warnings`
- `cargo test --workspace`
