# Preface

Knowledge base index for the `.workshop/yuescript/hooks/` directory, covering the hook scripts that install and health-check the `yue` in-project SDK.

# Directory

- `.workshop/yuescript/hooks/setup-project` - Downloads the standalone `yue` 0.34.1 binary from GitHub releases to `~/.local/bin/yue` with SHA-256 verification
- `.workshop/yuescript/hooks/check-health` - Verifies `~/.local/bin/yue --version` runs and reports health to `workshopctl`

# Important

- These hooks run as the workshop user; `yue` lands in `/home/workshop/.local/bin` (on PATH).
- The standalone binary is used instead of the `yue` snap because the snap's strict confinement prevents it from reading files under `/project`; the standalone binary has no such limitation.
- The `yue` version is pinned to `0.34.1` in `setup-project` via the `VERSION` variable and `SHA256` checksum, and mirrored in `.workshop/yuescript/sdk.yaml`; bumping requires editing both files together, with the `SHA256` variable to be manually updated.
- `check-health` reports `okay` only when the binary is executable and `--version` succeeds; otherwise it reports `error` with a short message via `workshopctl set-health`.
