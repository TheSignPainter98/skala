# `skala-graph`

`skala-graph` is a terminal UI for exploring the state of SKALA-controlled
reactor over time.

Note: this part of the project is entirely machine-generated. The same
standards of code-quality that the rest of the codebase attempt to adhere to do
not apply here. This isn't a critical component.

## Usage

```text
skala-graph <DB_PATH> [--reactor <NAME>]
```

- `<DB_PATH>` is the path to the SQLite database to open.
- `--reactor <NAME>` optionally preselects a reactor before the UI starts.

## Examples

Open the intended sample database:

```bash
cargo run -- /path/to/skala.db
```

Open a database and preselect a reactor:

```bash
cargo run -- /path/to/skala.db --reactor reactor_53
```

## Startup behaviour

- The database is opened read-only.
- Startup fails if the expected SKALA tables or columns are missing.
- If the database contains no reactors with events, startup fails.
- If `--reactor` is set and the name is unknown, startup fails and lists the
  known reactor names.
- Without `--reactor`, the first reactor in name order is selected at startup.
  You can switch reactors later inside the UI.

## Default view

The initial chart enables these metrics:

- `temperature`
- `actual_burn_rate`
- `target_burn_rate`
- `energy_production_rate`

All selected metrics share one chart. Values are normalised for plotting, while
raw values are still shown in the interface labels and status text.

## Controls

- `Tab`: switch focus between the reactor list and metric list
- `Up` / `Down`: move within the focused list
- `Space`: toggle the highlighted metric
- `Left`: hide all metrics
- `Right`: show all metrics
- `r`: reload the database from disk
- `q` or `Ctrl+C`: quit

## Help output

Show the built-in CLI help with:

```bash
cargo run -- --help
```
