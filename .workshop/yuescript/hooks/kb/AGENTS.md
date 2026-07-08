# Preface

Knowledge base index for the `.workshop/yuescript/hooks/` directory, covering the hook scripts that install and health-check the `yue` in-project SDK.

# Directory

- `.workshop/yuescript/hooks/setup-project` - Downloads the YueScript 0.34.1 source tarball from GitHub, builds `yue` from source with `make release`, and installs it to `~/.local/bin/yue` with SHA-256 verification
- `.workshop/yuescript/hooks/check-health` - Verifies `~/.local/bin/yue --version` runs and reports health to `workshopctl`

# Important

- These hooks run as the workshop user; `yue` lands in `/home/workshop/.local/bin` (on PATH).
- The compiled `yue` binary is installed to `/home/workshop/.local/bin` (on PATH). We build from source rather than using the prebuilt binary zip so that the project uses `.tar.gz` (for which build tools are already available) and gets a fresh build with all dependencies including the bundled Lua runtime.
- The `yue` version is pinned to `0.34.1` in `setup-project` via the `VERSION` variable and `SHA256` checksum, and mirrored in `.workshop/yuescript/sdk.yaml`; bumping requires editing both files together, with the `SHA256` variable to be manually updated.
- The source tarball is expanded into a temp directory, built with `make release` (which also compiles the bundled Lua dependency), and the `bin/release/yue` binary is installed.
- `check-health` reports `okay` only when the binary is executable and `--version` succeeds; otherwise it reports `error` with a short message via `workshopctl set-health`.
