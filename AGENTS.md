# Repository Guidelines

## Scope

These instructions apply to the whole repository, except vendored code under
`skala_server/vendor/**`. Do not modify vendored code unless the task explicitly
requires it.

Ignore any `AGENTS.md` files found inside vendored dependencies; they belong to
upstream packages and do not override these instructions. Scoped `AGENTS.md`
files under `skala_client/` and `skala_server/` add local guidance for those
parts of the project.

## Project Overview

- S.K.A.L.A. is a CC:Tweaked and Mekanism reactor control project. The client
  runs in ComputerCraft/Lua, the server records reactor state and asks an LLM
  advisor for advice, and the graph tool explores recorded SQLite data.
- The server is the source of truth for the HTTP API contract. Types that derive
  `quicktype::Quicktype` generate the client-facing type definitions.
- Reactor observations are stored as events with related reactor snapshots,
  advice records, and production targets. Historical data is used both for LLM
  context and for graphing.

## Repository Structure

- `skala_client/` contains YueScript source for the ComputerCraft client.
  Generated Lua files and `bin/` output are build artefacts; edit `.yue` files
  rather than generated Lua.
- `skala_server/` contains the Rust 2024 Axum/SQLx SQLite server workspace,
  migrations, quicktype support crates, advisor implementations, routes, and
  integration tests.
- `skala_graph/` contains the standalone Rust 2024 `ratatui` TUI for opening
  SKALA SQLite databases and graphing reactor metrics.
- `results/` stores recorded SQLite databases used as local samples and graph
  fixtures.

## Language And Tone

- Use British English in documentation, comments, prompts, error messages, and
  user-facing copy.
- Keep technical documentation formal and operational. The README has a playful
  project tone, but do not introduce humour into agent guidance or engineering
  instructions unless you are editing existing themed copy.

## Coding Standards

- New functions must include explicit type annotations where the language
  supports them.
- In Rust `Cargo.toml` files, declare direct dependencies with
  `default-features = false` and opt into the required features explicitly. Do
  not apply this rule to proc-macro crates written in this repository, for
  example `quicktype_macros`.
- Keep changes close to the surrounding style. Avoid broad refactors while
  making feature or bug-fix changes.
- Prefer small, behaviour-focused changes with matching tests when code changes
  affect server routes, advisor behaviour, generated type contracts, or client
  control flow.

## Platform Support

- The project is only guaranteed to work on Linux. Treat Windows, macOS, and
  other platforms as best-effort development environments unless a task
  explicitly expands the supported platform set.

## Verification

- Run checks that are relevant to the area changed. Use scoped `AGENTS.md` files
  for client and server commands.
- Test all changes using the `prek` command.
- For documentation-only changes, a focused Markdown review and
  `git diff --check` are enough.

## Workflow Safety

- Preserve user changes in a dirty worktree. Do not revert or overwrite work you
  did not make unless explicitly instructed.
