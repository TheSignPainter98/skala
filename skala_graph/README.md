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

Open a database and let it wait for reactor data and auto-reload:

```bash
cargo run -- /path/to/skala.db
```

## Startup behaviour

- The database is opened read-only.
- Startup fails if the expected SKALA tables or columns are missing.
- If the database contains no reactors with events, the app prints a startup
  message and waits until reactor event data appears.
- If `--reactor` is set and the name is unknown, startup fails and lists the
  known reactor names.
- Without `--reactor`, the first reactor in name order is selected at startup.
  You can switch reactors later inside the UI.
- If the database only contains one reactor, the reactor list is hidden and the
  metrics and chart panes use the full width.
- The app retries an automatic reload every 0.25 seconds. If a reload fails,
  the last good data remains visible and the error is shown in the status line
  until a later reload succeeds.

## Default view

The initial chart enables these metrics:

- `damage_percent`
- `energy_production_rate`
- `advice_new_target_burn_rate`
- `production_target_rate`

All selected metrics share one chart. Normalised plotting is the default.
Press `n` to switch to raw plotting, which keeps all visible selected metrics on
the same chart and uses one shared Y axis starting at zero and extending to
the highest raw value across the currently visible selected series. Raw values
are still shown in the interface labels and status text.

## Controls

- `Tab`: switch focus between the reactor list and metric list when multiple
  reactors are available
- `Up` / `Down`: move within the focused list
- `Space`: toggle the highlighted metric
- `Left`: hide all metrics
- `Right`: show all metrics
- `m`: toggle the metrics pane
- `n`: toggle between normalised and raw chart scaling
- `r`: reload the database from disk
- automatic reload runs every 0.25 seconds
- `q` or `Ctrl+C`: quit

## Help output

Show the built-in CLI help with:

```bash
cargo run -- --help
```
