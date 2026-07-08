# Preface

Comprehensive reference for Workshop, SDKcraft, SDKs, interfaces, CLI, architecture, and troubleshooting.

# Architecture

## System Components

- **Workshop CLI** - User interface; thin client communicating with workshopd
- **workshopd** - Daemon; manages workshop lifecycle, container operations, SDK installation, interface coordination, state persistence
- **workshopctl** - Helper; runs inside workshop; limited REST API for reporting and control
- **workshop socket** - `/var/lib/workshop/run/workshop.socket.untrusted` (in-workshop)
- **LXD daemon** - `/var/snap/lxd/common/lxd/unix.socket` (on host)
- **State database** - _state.json_; authoritative workshop metadata and configuration
- **ZFS storage** - Storage pool for containers, caches, snapshots
- **Interface policy** - Validates and enforces plug/slot connections
- **systemd** - workshopd service integration

## Daemon Components

- REST API server (v1)
- State engine
- Task runner
- State managers: workshops, SDKs, interfaces, commands, hooks
- Graceful shutdown with task completion
- Structured logging for telemetry

## REST API

**Trusted endpoints** (require socket credentials):
- `/v1/projects/{id}/workshops` - Workshop lifecycle
- `/v1/sdks` - SDK management
- `/v1/connections` - Interface connections
- `/v1/changes` - Change tracking
- `/v1/warnings` - Warnings/errors

**Untrusted endpoints** (accessible from workshops):
- `/v1/workshopctl` - Workshop control interface

## Storage

- Linux: ZFS
- WSL: Btrfs (automatic)
- Pool name: `workshop`

## User/ID Mapping

- Host user -> workshop user in container
- Default: `uid=1000`, `gid=1000`
- `$XDG_RUNTIME_DIR`: `/run/user/1000/`

## Runtime Launch Flow

```
CLI -> workshopd API -> Daemon -> TaskRunner
    -> LXD: Create container
    -> ZFS: Create root filesystem
    -> Install system SDK -> snapshot
    -> For each SDK:
    |   -> Install SDK -> run setup-base -> snapshot
    |   -> Connect interfaces
    -> Start container
    -> Health checks -> Ready
```

## Container Image Handling

- Ubuntu cloud images from https://cloud-images.ubuntu.com/releases/
- Caching on host
- Supported: Ubuntu 22.04 LTS, 24.04 LTS, 26.04 LTS

## Network

- Dedicated bridge: `workshopbr0`
- DNS domain: `workshop`

# Important

## Overview

**Workshop** is Canonical's development environment tool that creates isolated, consistent development environments using containers. It wraps **LXD** containers with SDKs (software development kits) to provide reproducible tools, runtimes, and dependencies.

Workshops are secure, fast, and composable development environments that come agent-ready. They wrap complex, error-prone workspaces into reliable and reproducible definitions of languages, libraries, and tooling.

**Key Components:**
- **Workshop**: Manages workshop lifecycle (launch, refresh, remove)
- **SDKcraft**: Builds and publishes SDKs
- **SDK**: Packaged development tools and dependencies
- **workshopd**: Backend daemon managing container lifecycle
- **workshopctl**: In-workshop helper for SDK hooks

**Core Principles:**
- **Immutability**: Workshops evolve predictably through definition files
- **Layering**: SDKs layer on top of base images using ZFS snapshots
- **Isolation**: Each workshop runs in its own LXD container with its own project
- **Interfaces**: Standardized resource sharing (GPU, mount, tunnel, etc.)
- **Persistence**: Mount interfaces preserve data across workshop refreshes

## Tutorial

### Part 1: Get Started

Install:
```bash
sudo snap install workshop
```

Create a workshop definition (e.g. `.workshop/nstarlark.yaml`):
```yaml
name: dev
base: ubuntu@24.04
sdks:
  - name: go
    channel: "1.26"
```

Launch:
```bash
workshop launch dev
workshop shell dev
```

Projects are directories containing workshop definitions. A `.workshop.lock` file establishes the relationship between project and workshop.

### Part 2: Work with Interfaces

Interfaces are standardized mechanisms for resource sharing. The main interfaces are:
- **Mount** - filesystem directories
- **Tunnel** - network service forwarding
- **GPU** - GPU pass-through
- **SSH** - SSH agent access
- **Camera** - camera/video capture
- **Desktop** - Wayland/X11 display
- **Custom Device** - arbitrary host devices

Auto-connect happens for mount, GPU, and tunnel interfaces (with same-name matching). Others require manual `workshop connect`.

### Part 3: Sketch SDKs

Sketch SDKs are throwaway local experiments:
```bash
workshop sketch-sdk
```

Created with `workshop sketch-sdk`, unavailable from SDK Store, unique to the workshop where created. Can be ejected as an in-project SDK at `~/.workshop/<name>/sdk.yaml`.

### Part 4: Craft SDKs

Full SDKs use SDKcraft:
```yaml
# sdkcraft.yaml
name: my-sdk
build-base: ubuntu@24.04
title: My SDK
summary: A short summary
description: |
  Detailed description
version: "1.0.0"
platforms:
  amd64:
    build-on: [amd64]
    build-for: [amd64]
plugs:
  cache:
    interface: mount
    workshop-target: /home/workshop/.cache
slots:
  venv:
    interface: mount
    workshop-source: /home/workshop/venv
parts:
  main:
    plugin: dump
    source: https://example.com/file.tar.gz
    source-type: tar
    organize:
      my-bin: bin/my-bin
    prime:
      - bin/my-bin
```

Build and publish:
```bash
sdkcraft init my-sdk
sdkcraft build
sdkcraft pack
sdkcraft upload my-sdk 1.0.0 stable
sdkcraft release my-sdk stable
```

## How-To Guides

### Customize Workshops

#### Add Actions to Workshops

In _workshop.yaml_:
```yaml
actions:
  lint: |
    golangci-lint run -c .golangci.yaml
  build: |
    go build ./...
```

Run:
```bash
workshop run workshop-name -- lint "arg1" "arg2"
```

Actions receive trailing arguments as `$@`, `$1`. They are **not** part of ZFS snapshots - parsed from definition each time.

#### Add Mounts

Mount interface plugs in SDK:
```yaml
plugs:
  cache:
    interface: mount
    workshop-target: /home/workshop/.cache
```

Remount to specific host directory:
```bash
workshop remount workshop/sdk:plug-name /path/to/host/dir
```

#### Forward Ports (Tunnel Interface)

Tunnel interface for network service forwarding.

**Plug (system SDK for host->workshop):**
```yaml
plugs:
  app:
    interface: tunnel
    endpoint: 8080
```

**Slot (regular SDK providing service):**
```yaml
slots:
  app:
    interface: tunnel
    endpoint: 8080
```

**Endpoint formats:**
- TCP: `127.0.0.1:8080/tcp` or `localhost:8080` or `8080`
- UDP: `127.0.0.1:8080/udp` or `8080/udp`
- Unix socket: `/run/service.sock` or `@abstract`
- IPv6: `'[::1]:8080/tcp'` or `ip6-localhost:8080`

**Auto-connect rules:**
- Plug in system SDK, slot in regular SDK
- Plug listens on loopback or Unix socket
- Name matches or wired via `connections:`

#### Move Projects Around

Move the project directory; Workshop updates automatically via project tracking. The `.workshop.lock` file must remain in the project directory. **Copying** creates independent workshops. For multiple workshops in one project, see Multi-Workshop Patterns under Core Concepts.

### Develop with Workshops

#### Connect VS Code to a Workshop

Add `vscode-remote` SDK, launch the workshop, then use the VS Code Remote-Containers extension to connect via SSH.

#### Run JetBrains Gateway

1. Expose a tunnel port on the host
2. Upload a public SSH key via an action
3. Connect JetBrains Gateway using the `workshop` user on the exposed port

#### Run JupyterLab in Browser

1. Launch workshop with appropriate SDK
2. Connect tunnel interface for port forwarding
3. Access via browser

#### Manage Python Environments

Use Python SDK with mount interface for cache persistence.

#### Run GitHub Actions Locally

Use workshop with GitHub Actions SDK and appropriate interfaces.

#### Run Workshops in GitHub Actions

Configure workshop environment in CI/CD workflow.

#### Use Workshops with AI Agents

Workshop provides:
- LLM-readable documentation at `/docs/llms.txt` and `/docs/llms-full.txt`
- Context7 integration for MCP server
- `use-workshop` skill for agentic operations
- `sdk-designer` skill for interactive SDK creation

#### Use Workshops with Git

Workshop integrates with Git via mount interface for repository access.

## Reference

### CLI Reference

#### Workshop CLI

**Lifecycle Commands:**
- `workshop launch <name>` - First-time launch; builds from scratch
- `workshop refresh <name>` - Apply definition changes; reuses unchanged SDK snapshots
- `workshop remove <name>` - Remove workshop completely
- `workshop restore <name>` - Roll back to last successful launch/refresh snapshot
- `workshop start <name>` - Start stopped workshop
- `workshop stop <name>` - Stop running workshop

**Customize:**
- `workshop sketch-sdk` - Create a temporary sketch SDK
- `workshop sketches` - List sketch SDKs

**Enumerate:**
- `workshop info <name>` - Inspect workshop details
- `workshop list [project]` - List workshops in project

**Track Changes:**
- `workshop changes <name>` - Review recent changes
- `workshop tasks <name>` - Review tasks in a change

**Manage Connections:**
- `workshop connect <plug>` - Connect a plug to a slot
- `workshop connections --all` - List all connections
- `workshop disconnect <plug>` - Disconnect a plug
- `workshop remount <plug> <path>` - Remount a mount plug to new source

**Run Commands:**
- `workshop actions` - List actions
- `workshop run <name> -- <action>` - Run an action
- `workshop exec <name> -- <cmd>` - Run command in workshop
- `workshop shell <name>` - Open interactive shell

**Warnings:**
- `workshop okay` - Acknowledge warnings
- `workshop warnings` - List warnings

**Common Options:**
- `--project <path>` - Specify project directory
- `--global` - Show all workshops across projects
- `--wait-on-error` - Pause in waiting state on error
- `--abort` - Abort waiting state
- `--continue` - Continue from waiting state

#### SDK CLI

- `sdk find <query>` - Search SDK Store
- `sdk info <name>` - Inspect SDK details
- `sdk list` - List installed SDKs

#### SDKcraft CLI

- `sdkcraft init` - Bootstrap new project
- `sdkcraft build` - Build entire pipeline
- `sdkcraft pack` - Pack SDK for installation
- `sdkcraft try` - Install SDK to try area locally
- `sdkcraft login` - Authenticate
- `sdkcraft register <name>` - Claim SDK name
- `sdkcraft create-track <name> <track>` - Create track
- `sdkcraft upload <name> <version> <channel>` - Upload artifact
- `sdkcraft release <name> <channel>` - Release to channel

#### Workshopctl CLI

In hook context:
```bash
workshopctl set-health okay
workshopctl set-health waiting "<reason>"
workshopctl set-health error "<reason>"
```

### Definition Files

#### Workshop Definition File

```yaml
name: workshop-name          # required
base: ubuntu@24.04           # required
sdks:                        # list of SDK entries
  - name: go
    channel: "1.26"
  - name: custom-sdk
    slot: true
plugs:                       # additional plugs gifted to SDKs
slots:                       # additional slots gifted to SDKs
connections:                 # explicit interface connections
  - plug: consumer-sdk:plug-name
    slot: provider-sdk:slot-name
actions:
  lint: |
    golangci-lint run -c .golangci.yaml
```

- `name` (string): Workshop name (required)
- `base` (string): Base image (e.g. `ubuntu@24.04`) (required)
- `sdks` (array): List of SDK entries
- `plugs` (object): Additional plugs gifted to SDKs
- `slots` (object): Additional slots gifted to SDKs
- `connections` (array): Explicit interface connections
- `actions` (object): Named shell scripts

#### SDKcraft Definition File

```yaml
name: sdk-name                 # required
summary: Short summary         # required
description: |                 # required, multiline
  Detailed description
version: "1.25.1"              # required
build-base: ubuntu@24.04       # required
title: Short title             # optional
platforms:                     # optional
  amd64:
    build-on: [amd64]
    build-for: [amd64]
slot: true                     # optional - slot for SDK Store
plugs:                         # optional
  cache:
    interface: mount
    workshop-target: /home/workshop/.cache
slots:                         # optional
  venv:
    interface: mount
    workshop-source: /home/workshop/venv
parts:                         # required
  main-part:
    plugin: dump
    source: https://example.com/file.tar.gz
    source-type: tar
    organize:
      my-bin: bin/my-bin
    prime:
      - bin/my-bin
package-repositories:          # optional
adopt-info: part-name          # optional
source-code: "https://..."     # optional
contact: "author@example.com"  # optional
issues: "https://..."          # optional
```

#### SDK Definition File

```yaml
name: go
version: "1.26.0"
title: Go SDK
summary: The Go programming language
base: ubuntu@24.04
plugs:
  cuda:
    interface: mount
    workshop-target: /usr/local/cuda/lib64
slots:
  images:
    interface: mount
    workshop-source: $SDK/images
```

### Workshop Status States

- **Off** - Container does not exist
- **Ready** - Container is running and ready
- **Stopped** - Running but stopped
- **Pending** - Being updated/changed
- **Waiting** - Paused for interactive debugging
- **Error** - Nonfunctional state due to an error

### Status Transitions

```
Off ── launch ──→ Ready
              └─ launch (error) ──→ Error or Waiting

Ready ── refresh ──→ Ready (success)
          ── refresh (error) ──→ Error or Waiting

Ready ── stop ──→ Stopped ── start ──→ Ready

Ready ── restore ──→ Ready (discards runtime drift)
```

## Core Concepts

### Workshops

#### Workshop Concepts

A workshop is a development environment running in a container, defined by a YAML file describing base OS, installed SDKs, interface connections, and named actions.

**Container Layout:**
- `/` - Root filesystem (ZFS dataset)
- `/project/` - Mounted project directory from host
- `/var/lib/workshop/` - SDK volumes, state volumes, sockets

#### Project Concepts

A project is a directory containing one or more workshop definitions.

**Single Workshop:**
```
my-project/
├── workshop.yaml
└── .workshop.lock
```

**Multiple Workshops:**
```
my-project/
├── .workshop/
│   ├── frontend.yaml
│   ├── backend.yaml
│   └── common-tools/sdk.yaml
├── web/
└── api/
```

#### Workshop Lifecycle

**Launch** (first-time build):
1. Creates container from base image
2. Applies SDK layer by layer
3. ZFS snapshot after each SDK
4. `setup-base` hooks run
5. Interface connections established
6. `setup-project` hooks run
7. Health checks verify readiness

**Refresh** (apply changes):
- Only changed SDKs reinstalled
- Unchanged SDKs restored from snapshots (fast)
- `save-state` hooks run before rebuild; `restore-state` after
- Changing any SDK above causes all below to re-run `setup-base`
- Drops manual runtime connections

**Restore** (discard drift):
- Rolls filesystem back to last snapshot
- Discards all runtime changes
- Re-evaluates auto-connect; drops runtime connections

#### Multi-Workshop Patterns

Two patterns:

1. **One project, multiple workshops:** definitions in `.workshop/` subdirectory with one file per workshop:
   ```
   my-project/
   ├── .workshop/
   │   ├── frontend.yaml
   │   ├── backend.yaml
   │   └── common-tools/
   │       └── sdk.yaml  # In-project SDK
   ├── web/
   └── api/
   ```
2. **Multiple projects, one workshop each:** via Git Worktrees with one _workshop.yaml_ per project:
   ```
   feature-branch/
   └── workshop.yaml
   main-branch/
   └── workshop.yaml
   ```

Cross-workshop networking: two independent tunnels bridged through the host.

#### Changes and Tasks

- `workshop changes <name>` - Review recent changes since last launch/refresh
- `workshop tasks <name>` - Review tasks within a change

### SDKs

#### SDK Concepts

SDKs package tools, libraries, and configurations for workshops. They provide pre-packaged development environments, runtime hooks, and interface plugs/slots.

**Types:**
- **System SDK** - Automatically installed; exposes host resources through slots
- **Regular SDKs** - From SDK Store with channel versioning
- **In-Project SDKs** - Custom, project-specific SDKs
- **Sketch SDK** - Transient, local experiment (`workshop sketch-sdk`)

**Sketch SDK** is reserved name, always installed last, doesn't carry persistent data.

#### System SDK

Every workshop contains a special system SDK that:
- Exposes host system resources through slots
- Is automatically installed first during launch, removed last during remove
- Cannot be installed from SDK Store
- Cannot have additional content beyond resource exposure
- Can define plugs for workshop-internal resources

#### In-Project SDKs

Stored at `~/.workshop/<name>/sdk.yaml`. Shared via version control. Skip the build step.

#### SDK Parts

Parts modularize SDKs into discrete components:
```yaml
parts:
  ollama:
    plugin: dump
    source: https://github.com/ollama/ollama/releases/download/v0.9.6/ollama-linux-amd64.tgz
    source-type: tar
  user-service:
    plugin: dump
    source: ollama.service
    source-type: file
```

**Best practices:**
- Organize around functional boundaries
- Binary artifacts as parts for pinned versions
- Debian packages installed in hooks

#### Parts vs Hooks
- **Parts**: Pin versions, custom builds, unavailable tools
- **Hooks**: Debian packages, dynamic installation, runtime setup

#### Runtime Hooks

Hooks are bash scripts that extend workshop behavior at specific points.

- `setup-base` (root): System-level preparation. Runs at SDK install/refresh, before project mount.
- `setup-project` (workshop user): Project-specific setup. Runs after project mount and auto-connect.
- `check-health` (root): Reports SDK health. Runs after `setup-project` (or `restore-state` on refresh).
- `save-state` (root): Persists data to `$SDK_STATE_DIR`. Runs during refresh, before rebuild (old revision).
- `restore-state` (root): Restores data from `$SDK_STATE_DIR`. Runs during refresh, after all `setup-project` (new revision).

**Execution contract:**
- Non-interactive bash login session
- `errexit` and `pipefail` set
- With `--verbose`, `xtrace` also set

**Health Checking:**
```bash
workshopctl set-health okay
workshopctl set-health waiting "<message>"
workshopctl set-health error "<message>"
```

#### SDK Lifecycle

1. **Sketch** - Throwaway local experiment (`workshop sketch-sdk`)
2. **In-Project SDK** - Shared via version control (`workshop sketch-sdk --eject`)
3. **SDKcraft Project** - Full build with parts, hooks, platforms, tests
4. **Publish** - Register, upload, release on SDK Store
5. **Consume** - Add to _workshop.yaml_ and pick a channel

#### SDK Channels

Structure: `track/risk[/branch]`

- Track - Multiple parallel versions; use semantic version or `latest`
- Risk - Maturity: `stable`, `candidate`, `beta`, `edge`
- Branch - Short-lived subdivision; auto-closed after 30 days

#### Best Practices

- System services: use systemd service files
- Environment variables: `/etc/profile.d/sdk.sh` (system-wide), `~/.profile` (user-specific)
- Avoid `~/.bashrc` (shell-specific)
- Health checks: test relevant features, provide specific error codes, run quickly

#### SDKs Versus Dockerfiles

Workshop replaces Docker layering with ZFS snapshots. Instead of `ENV` and `VOLUME`, Workshop uses explicit interface plugs/slots for resource sharing.

### Interfaces

#### Interface Concepts

SDKs connect to resources via **plugs** (consumers) and **slots** (providers).

**Slots** provide capabilities (mount directory, tunnel endpoint, GPU, etc.)
**Plugs** consume capabilities (must stay declared even if no slot connected - optional)

#### Wiring Mechanisms

1. **Inline Plug Bindings** (within `sdks:`):
```yaml
sdks:
  - name: consumer-sdk
    plugs:
      tools:
        bind: provider-sdk:tools
```

2. **Top-level Connections**:
```yaml
connections:
  - plug: consumer-sdk:tools
    slot: provider-sdk:slot
```

#### Auto-Connection Behavior

- Mount - Yes (to system SDK slots by default)
- GPU - Yes
- Tunnel - Only host->workshop, loopback/Unix socket, same name
- Camera - No (manual)
- Custom Device - No (manual)
- Desktop - No (manual)
- SSH - No (manual)

#### Camera Interface

```yaml
plugs:
  camera:
    interface: camera
```
Manual connection required for security:
```bash
workshop connect my-workspace/camera-sdk:camera
```

#### Custom Device Interface

```yaml
plugs:
  input-device:
    interface: custom-device
    subsystem: input
```

Query subsystem:
```bash
udevadm info --query=property --property=SUBSYSTEM /dev/input/event0
```

#### Desktop Interface

Provides Wayland/X11 display access. Environment variables set on connection: `WAYLAND_DISPLAY`, `DISPLAY`, `XDG_SESSION_TYPE`, `XAUTHORITY`.

#### GPU Interface

```yaml
plugs:
  gpu:
    interface: gpu
```
Auto-connects at launch/refresh if plug matches slot.

#### Mount Interface

Mount plug:
```yaml
plugs:
  cache:
    interface: mount
    workshop-target: /home/workshop/.cache
```

Slot (workshop or host):
```yaml
slots:
  data:
    interface: mount
    workshop-source: /home/workshop/data
```

Remount:
```bash
workshop remount my-workspace/mount-sdk:cache ~/.local/cache/
```

#### SSH Interface

```yaml
plugs:
  ssh-agent:
    interface: ssh-agent
```
Manual connection sets `$SSH_AUTH_SOCK` for workshop user.

## Troubleshooting

### Workshop Fails to Launch
- Check base image availability
- Verify workspace permissions
- Review `workshop changes` and `workshop tasks` output
- Use `--verbose` for detailed logging

### Interface Connection Failures
- Check plug/slot compatibility
- Verify interface policy allows auto-connection
- Use `workshop connections --all`
- Manual connect: `workshop connect workspace-name/sdk-name:plug-name`

### Storage Issues
- ZFS pool minimum is 5 GiB
- Use `lxc storage` to tune ZFS pool
- Don't use `zfs` or `btrfs` utilities directly

### SDK Hook Failures
- Hooks exit on non-zero code (errexit, pipefail)
- With `--verbose`, xtrace also set
- Use `workshopctl set-health` to report status
- `setup-base` runs as root; `setup-project` runs as workshop user

### Workshop Drift
- Use `workshop restore` to discard drift
- Review changes since last successful launch/refresh

### Connection Persistence
- Connections in _workshop.yaml_ survive refresh
- Runtime connections (via `workshop connect`) drop on refresh
- Manual disconnects survive refresh and restore

## Additional Resources

### Workshop and AI Agents

**LLM-Readable Documentation:**
- `/docs/llms.txt` - Index of pages
- `/docs/llms-full.txt` - Full concatenated Markdown
- Pages: `<url>.md` (e.g., _/docs/reference/workshop-cli.md_)

**Context7:** MCP server for agent integration at https://context7.com/canonical/workshop

**Skills:**
- `use-workshop` skill for agentic operations: https://github.com/canonical/use-workshop-skill
- `sdk-designer` skill for interactive SDK creation: https://github.com/canonical/template-sdk
