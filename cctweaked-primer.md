# CC: Tweaked Lua Environment Primer

A comprehensive reference of all functions, types, peripherals, modules, generic peripherals, and events available in CC: Tweaked's Lua environment, sourced from the official documentation at [tweaked.cc](https://tweaked.cc/).

> **Note on British English:** CC: Tweaked provides `colours` as an alias of `colors` (with `grey`/`lightGrey` instead of `gray`/`lightGray`), and `serialise`/`unserialise` as aliases of `serialize`/`unserialize`. The documentation below favours British spellings where both exist.

Last updated: 2026-07-30

---

## Table of Contents

1. [Global Environment (`_G`)](#1-global-environment-_g)
2. [Core Modules](#2-core-modules)
   - [`os`](#os) · [`fs`](#fs) · [`term`](#term) · [`textutils`](#textutils)
   - [`colors` / `colours`](#colors--colours) · [`redstone`](#redstone) · [`paintutils`](#paintutils)
   - [`vector`](#vector) · [`turtle`](#turtle) · [`commands`](#commands)
   - [`http`](#http) · [`settings`](#settings) · [`help`](#help)
   - [`shell`](#shell) · [`multishell`](#multishell) · [`gps`](#gps)
   - [`rednet`](#rednet) · [`parallel`](#parallel) · [`window`](#window)
   - [`io`](#io) · [`keys`](#keys) · [`disk`](#disk)
   - [`peripheral`](#peripheral) · [`pocket`](#pocket)
3. [Library Modules (`cc.*`)](#3-library-modules-cc)
4. [Peripherals](#4-peripherals)
5. [Generic Peripherals](#5-generic-peripherals)
6. [Events](#6-events)

---

## 1. Global Environment (`_G`)

The global environment contains built-in Lua functions and CC: Tweaked-specific extensions defined in `bios.lua`.

### CC: Tweaked Global Functions

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `sleep` | `time? number` (default 0) | — | Pauses execution for `time` seconds (rounded to 0.05s increments). Yields. Discards events during sleep. |
| `write` | `text: string` | `number` | Writes text without newline, wrapping if necessary. |
| `print` | `...: any` | `number` | Prints values separated by spaces with trailing newline. |
| `printError` | `...: any` | — | Prints values in red with newline. |
| `read` | `replaceChar? string`, `history? table`, `completeFn? function(string):{string...}`, `default? string` | `string` | Reads user input. `replaceChar` masks input (e.g. `"*"`); `completeFn` provides tab-completion; `default` sets initial text. |

### Global Constants

| Constant | Type | Description |
|---|---|---|
| `_HOST` | `string` | Version string: `"ComputerCraft X.Y.Z (Minecraft A.B.C)"`. |
| `_CC_DEFAULT_SETTINGS` | `string` | Default computer settings from config (comma-separated `key=value` pairs). |

---

## 2. Core Modules

### `os`

Interacts with the current computer.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `pullEvent` | `filter? string` | `string, ...` | Waits for an event matching `filter`. Stops program on `terminate`. |
| `pullEventRaw` | `filter? string` | `string, ...` | Like `pullEvent` but does not handle `terminate`. |
| `version` | — | `string` | CraftOS version (e.g. `"CraftOS 1.9"`). |
| `run` | `env: table`, `path: string`, `...` | `boolean` | Runs a program at `path` with given environment and args. |
| `queueEvent` | `name: string`, `...` | — | Adds an event to the event queue. |
| `startTimer` | `time: number` | `number` | Starts a timer (rounded to 0.05s). Fires `timer` event. |
| `cancelTimer` | `token: number` | — | Cancels a timer. |
| `setAlarm` | `time: number` | `number` | Sets alarm at in-game time 0–24. Fires `alarm` event. |
| `cancelAlarm` | `token: number` | — | Cancels an alarm. |
| `shutdown` | — | — | Shuts down immediately. |
| `reboot` | — | — | Reboots immediately. |
| `getComputerID` | — | `number` | Computer's ID. |
| `getComputerLabel` | — | `string?` | Computer's label or `nil`. |
| `setComputerLabel` | `label? string` | — | Sets label (pass `nil` to clear). |
| `clock` | — | `number` | Uptime in seconds. |
| `time` | `locale? string\|table` | `number` | Current time: `ingame` (default), `utc`, or `local`. Accepts `os.date("*t")` table → UNIX timestamp. |
| `day` | `locale? string` | `number` | Day count: `ingame` (default), `utc`, or `local`. |
| `epoch` | `locale? string` | `number` | Milliseconds since epoch: `ingame` (default), `utc`, or `local`. 1 real sec = 72000 in-game ms. |
| `date` | `format? string`, `time? number` | `string\|table` | Date/time formatting. `"*t"` returns table with `year`, `month`, `day`, `hour`, `min`, `sec`, `wday`, `yday`, `isdst`. |

### `fs`

Filesystem operations. All paths are absolute.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `complete` | `path: string`, `location: string`, `include_files? boolean` (default true), `include_dirs? boolean` (default true) | `{string...}` | Path completion candidates. |
| `find` | `path: string` | `{string...}` | Wildcard file search (`?` single char, `*` multiple chars). |
| `isDriveRoot` | `path: string` | `boolean` | True if path is a mount point. |
| `list` | `path: string` | `{string...}` | List directory contents. |
| `combine` | `path: string`, `...: string` | `string` | Join path parts. |
| `getName` | `path: string` | `string` | File name portion. |
| `getDir` | `path: string` | `string` | Parent directory. |
| `getSize` | `path: string` | `number` | File size in bytes. |
| `exists` | `path: string` | `boolean` | Path exists. |
| `isDir` | `path: string` | `boolean` | Path is a directory. |
| `isReadOnly` | `path: string` | `boolean` | Path is read-only. |
| `makeDir` | `path: string` | — | Create directory and parents. |
| `move` | `path: string`, `dest: string` | — | Move file/directory. |
| `copy` | `path: string`, `dest: string` | — | Copy file/directory. |
| `delete` | `path: string` | — | Delete file/directory. |
| `open` | `path: string`, `mode: string` | `Handle?` | Open file. Modes: `r`, `w`, `a`, `r+`, `w+`, `b` suffix for binary. |
| `getDrive` | `path: string` | `string` | Mount name (`hdd`, `rom`, etc.). |
| `getFreeSpace` | `path: string` | `number` | Free space on drive. |
| `getCapacity` | `path: string` | `number` | Drive capacity. |
| `attributes` | `path: string` | `table` | `{size, isDir, isReadOnly, created, modified}`. |

**Types:** `Handle` — file handle with `read([format])`, `readAll()`, `readLine()`, `write(data)`, `writeLine(data)`, `flush()`, `close()`, `seek([whence[, offset]])`.

### `term`

Terminal output and drawing.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `nativePaletteColour` | `colour: number` | `r, g, b` | Default palette RGB (0–1) for a colour. |
| `write` | `text: string` | — | Write text at cursor. |
| `scroll` | `y: number` | — | Scroll up (positive) or down (negative). |
| `getCursorPos` | — | `x, y` | Cursor position. |
| `setCursorPos` | `x: number`, `y: number` | — | Set cursor position. |
| `getCursorBlink` | — | `boolean` | Cursor is blinking. |
| `setCursorBlink` | `blink: boolean` | — | Set cursor blink. |
| `getSize` | — | `width, height` | Terminal dimensions. |
| `clear` | — | — | Clear with background colour. |
| `clearLine` | — | — | Clear current line. |
| `getTextColour` | — | `number` | Current text colour. |
| `setTextColour` | `colour: number` | — | Set text colour. |
| `getBackgroundColour` | — | `number` | Current background colour. |
| `setBackgroundColour` | `colour: number` | — | Set background colour. |
| `isColour` | — | `boolean` | Supports colour output. |
| `blit` | `text: string`, `textColour: string`, `backgroundColour: string` | — | Write text with per-char colours (hex `0`–`f`). |
| `setPaletteColour` | `colour: number`, `r: number`, `g: number`, `b: number` | — | Set palette entry (r,g,b: 0–1). |
| `getPaletteColour` | `colour: number` | `r, g, b` | Get palette entry. |
| `redirect` | `target: Redirect` | `Redirect?` | Redirect output to another terminal. |
| `current` | — | `Redirect` | Current terminal object. |
| `native` | — | `Redirect` | Native terminal object. |

**Types:** `Redirect` — terminal redirect object (same methods as `term`). British aliases (`isColour`/`isColor`) are also provided.

### `textutils`

String and data formatting.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `slowWrite` | `text: string`, `rate? number` (default 20) | — | Character-by-character write. |
| `slowPrint` | `text: string`, `rate? number` (default 20) | — | Character-by-character print. |
| `formatTime` | `time: number`, `24h? boolean` | `string` | Format time (e.g. `"6:30 PM"`). |
| `pagedPrint` | `text: string`, `free_lines? number` | `number` | Print with paging. |
| `tabulate` | `...: {string...} \| number` | — | Print structured tables; number args set colour. |
| `pagedTabulate` | `...: {string...} \| number` | — | Tabulate with paging. |
| `serialize` | `t: any`, `opts?: {compact?: boolean, allow_repetitions?: boolean}` | `string` | Lua text serialisation. |
| `unserialize` | `s: string` | `any?` | Reconstruct from serialised string. |
| `serializeJSON` | `t: any`, `opts?: {nbt_style?: boolean, unicode_strings?: boolean, allow_repetitions?: boolean}` | `string` | JSON serialisation. |
| `unserializeJSON` | `s: string`, `opts?: {nbt_style?: boolean, parse_null?: boolean, parse_empty_array?: boolean}` | `any?` | JSON parsing. |
| `urlEncode` | `str: string` | `string` | URL-encode a string. |
| `complete` | `text: string`, `env? table` | `{string...}` | Complete a partial Lua expression; appends `.` for tables, `(` for functions. |

**Constants:** `empty_json_array` (empty JSON array vs object), `json_null` (JSON null value).

### `colors` / `colours`

See full colour table below. `colours` is the British alias (`grey`, `lightGrey`).

| Constant | Value | Blit | Hex | RGB |
|---|---|---|---|---|
| `white` | 1 | `0` | `#F0F0F0` | 240,240,240 |
| `orange` | 2 | `1` | `#F2B233` | 242,178,51 |
| `magenta` | 4 | `2` | `#E57FD8` | 229,127,216 |
| `lightBlue` | 8 | `3` | `#99B2F2` | 153,178,242 |
| `yellow` | 16 | `4` | `#DEDE6C` | 222,222,108 |
| `lime` | 32 | `5` | `#7FCC19` | 127,204,25 |
| `pink` | 64 | `6` | `#F2B2CC` | 242,178,204 |
| `gray`/`grey` | 128 | `7` | `#4C4C4C` | 76,76,76 |
| `lightGray`/`lightGrey` | 256 | `8` | `#999999` | 153,153,153 |
| `cyan` | 512 | `9` | `#4C99B2` | 76,153,178 |
| `purple` | 1024 | `a` | `#B266E5` | 178,102,229 |
| `blue` | 2048 | `b` | `#3366CC` | 51,102,204 |
| `brown` | 4096 | `c` | `#7F664C` | 127,102,76 |
| `green` | 8192 | `d` | `#57A64E` | 87,166,78 |
| `red` | 16384 | `e` | `#CC4C4C` | 204,76,76 |
| `black` | 32768 | `f` | `#111111` | 17,17,17 |

**Functions:** `combine(...: number): number`, `subtract(colors: number, ...: number): number`, `test(colors: number, color: number): boolean`, `packRGB(r: number, g: number, b: number): number`, `unpackRGB(rgb: number): r, g, b`, `toBlit(color: number): string`, `fromBlit(hex: string): number`. (`rgb8` is deprecated.)

### `redstone`

Also accessible as `rs`. Valid sides: `"top"`, `"bottom"`, `"left"`, `"right"`, `"front"`, `"back"`.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `getSides` | — | `{string...}` | Six valid sides. |
| `setOutput` | `side: string`, `on: boolean` | — | Binary output (strength 15 when on). |
| `getOutput` | `side: string` | `boolean` | Current output state. |
| `getInput` | `side: string` | `boolean` | Current input state. |
| `setAnalogOutput` | `side: string`, `value: number` (0–15) | — | Analogue output strength. |
| `getAnalogOutput` | `side: string` | `number` (0–15) | Output strength. |
| `getAnalogInput` | `side: string` | `number` (0–15) | Input strength. |
| `setBundledOutput` | `side: string`, `output: number` | — | Bundled cable output (colour bitmask). |
| `getBundledOutput` | `side: string` | `number` | Bundled output. |
| `getBundledInput` | `side: string` | `number` | Bundled input. |
| `testBundledInput` | `side: string`, `mask: number` | `boolean` | Test if colours in mask are set. |

### `paintutils`

Drawing primitives. **Warning:** These may change cursor position and background colour.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `parseImage` | `image: string` | `table` | Parse image string. |
| `loadImage` | `path: string` | `table?` | Load image from file. |
| `drawPixel` | `x, y: number`, `colour? number` | — | Draw a pixel. |
| `drawLine` | `startX, startY, endX, endY: number`, `colour? number` | — | Draw a line. |
| `drawBox` | `startX, startY, endX, endY: number`, `colour? number` | — | Unfilled box outline. |
| `drawFilledBox` | `startX, startY, endX, endY: number`, `colour? number` | — | Filled box. |
| `drawImage` | `image: table`, `x, y: number` | — | Draw an image. |

### `vector`

3D vector math. `v1 + v2`, `v1 - v2`, `v * n`, `v / n` operators supported.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `new` | `x, y, z: number` | `Vector` | Construct vector. |

**Methods:** `add(o)`, `sub(o)`, `mul(factor)`, `div(factor)`, `unm()`, `dot(o): number`, `cross(o): Vector`, `length(): number`, `normalize(): Vector`, `round(tolerance?: number): Vector`, `tostring(): string`, `equals(other): boolean`.

### `turtle`

Movement (consumes fuel unless unlimited): `forward()`, `back()`, `up()`, `down()`, `turnLeft()`, `turnRight()` — each returns `boolean, string?`.

Block interaction: `dig(side?)`, `digUp(side?)`, `digDown(side?)` — returns `boolean, string?`. `side` ∈ `{"left", "right"}` for tool selection.

Placement: `place(text?)`, `placeUp(text?)`, `placeDown(text?)` — returns `boolean, string?`.

Attacking: `attack(side?)`, `attackUp(side?)`, `attackDown(side?)` — returns `boolean, string?`.

Detection: `detect()`, `detectUp()`, `detectDown()` — returns `boolean`.

Comparison: `compare()`, `compareUp()`, `compareDown()` — returns `boolean`.

Inspection: `inspect()`, `inspectUp()`, `inspectDown()` — returns `boolean, table|string`.

Inventory: `select(slot: number)`, `getItemCount(slot?: number): number`, `getItemSpace(slot?: number): number`, `compareTo(slot: number): boolean`, `transferTo(slot: number, count?: number): boolean`, `getSelectedSlot(): number`, `getItemDetail(slot?: number, detailed?: boolean): table?`.

Item transfer: `drop(count?: number)`, `dropUp(count?: number)`, `dropDown(count?: number)`, `suck(count?: number)`, `suckUp(count?: number)`, `suckDown(count?: number)` — each returns `boolean, string?`.

Fuel: `getFuelLevel(): number|"unlimited"`, `getFuelLimit(): number|"unlimited"`, `refuel(count?: number): boolean, string?`.

Upgrades: `equipLeft(): boolean, string?`, `equipRight(): boolean, string?`, `getEquippedLeft(): table?`, `getEquippedRight(): table?`.

Crafting: `craft(limit?: number): boolean, string?`.

### `commands` (command computers only)

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `exec` | `command: string` | `boolean, {string...}, number?` | Execute command synchronously. |
| `execAsync` | `command: string` | `number` | Async execute; fires `task_complete`. |
| `list` | `...: string` | `{string...}` | List available commands. |
| `getDimension` | — | `string` | Current dimension. |
| `getBlockPosition` | — | `x, y, z` | Computer position. |
| `getBlockInfos` | `minX,Y,Z, maxX,Y,Z: number`, `dimension?: string` | `{table...}` | Block info for region (max 4096 blocks). |
| `getBlockInfo` | `x, y, z: number`, `dimension?: string` | `table` | Block info. |
| `getEntities` | `selector: string` | `{table...}` | Entities matching selector. |

`commands.native` — raw API without helpers. `commands.async` — async wrappers (`commands.async.setblock(...)`).

### `http`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `get` | `url: string`, `headers? {string=string}`, `binary? boolean` | `Response?` / `nil, string, Response?` | Synchronous GET. |
| `post` | `url: string`, `body: string`, `headers?`, `binary?` | `Response?` / `nil, string, Response?` | Synchronous POST. |
| `request` | `url, body?, headers?, binary?` | — | Async HTTP; fires `http_success`/`http_failure`. |
| `checkURL` | `url: string` | `true` / `false, string` | Synchronous URL validation. |
| `checkURLAsync` | `url: string` | `true` / `false, string` | Async URL check; fires `http_check`. |
| `websocket` | `url: string`, `headers?` | `Websocket?` / `false, string` | Synchronous websocket. |
| `websocketAsync` | `url: string`, `headers?` | — | Async websocket; fires `websocket_success`/`websocket_failure`. |

All accept table form: `{ url, body?, headers?, binary?, method?, redirect?, timeout? }`.

**Types:**
- `Response` — extends `Handle`. Methods: `getResponseCode(): number, string`, `getResponseHeaders(): {string=string}`.
- `Websocket` — methods: `receive(timeout?): string, boolean` / `nil, string`, `send(message: string, binary?: boolean)`, `close()`, `getResponseHeaders(): {string=string}`.

### `settings`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `define` | `name: string`, `options?: {description?, default?, type?}` | — | Define setting. `type` ∈ `"number"`, `"string"`, `"boolean"`, `"table"`. |
| `undefine` | `name: string` | — | Remove definition. |
| `set` | `name: string`, `value: any` | — | Set value (serialisable, not `nil`). Must call `save` to persist. |
| `get` | `name: string`, `default?: any` | `any` | Get value (uses defined default if unset). |
| `getDetails` | `name: string` | `table` | `{description, default, type, value}`. |
| `unset` | `name: string` | — | Reset to default. Fires `setting_changed`. |
| `clear` | — | — | Reset all settings. |
| `getNames` | — | `{string...}` | All defined names (sorted). |
| `load` | `path?: string` (default `".settings"`) | `boolean` | Load from file. |
| `save` | `path?: string` (default `".settings"`) | `boolean` | Save to file. |

### `help`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `path` | — | `string` | Colon-separated help paths. |
| `setPath` | `newPath: string` | — | Set help paths. |
| `lookup` | `topic: string` | `string?` | Find help file path. |
| `topics` | — | `{string...}` | All topics. |
| `completeTopic` | `prefix: string` | `{string...}` | Complete topic prefix. |

### `shell`

Provides access to CraftOS's command line interface. Not a "true" API—it is a standard program that injects its API into programs it launches.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `execute` | `command: string`, `...: string` | `boolean` | Run a program with arguments passed verbatim (no parsing). |
| `run` | `...: string` | `boolean` | Run a program with concatenated, parsed arguments. |
| `exit` | — | — | Exit the current shell. |
| `dir` | — | `string` | Current working directory. |
| `setDir` | `dir: string` | — | Set working directory. |
| `path` | — | `string` | Colon-separated program paths. |
| `setPath` | `path: string` | — | Set program paths. |
| `resolve` | `path: string` | `string` | Resolve relative path to absolute. |
| `resolveProgram` | `command: string` | `string?` | Resolve a program name to its path. |
| `programs` | `include_hidden?: boolean` | `{string...}` | List all programs on the path. |
| `complete` | `sLine: string` | `{string...}?` | Complete a shell command line. |
| `completeProgram` | `program: string` | `{string...}` | Complete a program name. |
| `setCompletionFunction` | `program: string`, `complete: function` | — | Set tab-completion for a program. |
| `getCompletionInfo` | — | `{string={fnComplete=function}}` | All completion functions. |
| `getRunningProgram` | — | `string` | Path to the currently running program. |
| `setAlias` | `command: string`, `program: string` | — | Add an alias. |
| `clearAlias` | `command: string` | — | Remove an alias. |
| `aliases` | — | `{string=string}` | Current aliases. |
| `openTab` | `...: string` | `number` | Open a new multishell tab. |
| `switchTab` | `id: number` | — | Switch to a multishell tab. |

### `multishell`

Runs multiple programs simultaneously with a tab bar. Each process has an ID corresponding to its tab position; IDs are not constant over a program's run.

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `getFocus` | — | `number` | Currently visible process index. |
| `setFocus` | `n: number` | `boolean` | Switch to a process by index. |
| `getTitle` | `n: number` | `string?` | Title of a process. |
| `setTitle` | `n: number`, `title: string` | — | Set a process title. |
| `getCurrent` | — | `number` | Currently executing process index. |
| `launch` | `env: table`, `path: string`, `...` | `number` | Start a new process. |
| `getCount` | — | `number` | Number of processes.

### `gps`

| Constant | Value | Description |
|---|---|---|
| `CHANNEL_GPS` | 65534 | Channel for GPS requests and responses. |

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `locate` | `timeout?: number` (default 2), `debug?: boolean` (default false) | `number, number, number` / `nil` | Get this computer's x, y, z position. |

### `rednet`

| Constant | Value | Description |
|---|---|---|
| `CHANNEL_BROADCAST` | 65535 | Broadcast channel. |
| `CHANNEL_REPEAT` | 65533 | Repeat channel. |
| `MAX_ID_CHANNELS` | 65500 | Reserved channels for computer IDs. |

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `open` | `modem: string` | — | Open a modem for rednet. |
| `close` | `modem?: string` | — | Close a modem. |
| `isOpen` | `modem?: string` | `boolean` | Check if rednet is open. |
| `send` | `recipient: number`, `message: any`, `protocol?: string` | `boolean` | Send a message to a specific computer. |
| `broadcast` | `message: any`, `protocol?: string` | — | Broadcast a message. |
| `receive` | `protocol_filter?: string`, `timeout?: number` | `number, any, string?` / `nil` | Wait for a message. |
| `host` | `protocol: string`, `hostname: string` | — | Register as a host. |
| `unhost` | `protocol: string` | — | Stop hosting a protocol. |
| `lookup` | `protocol: string`, `hostname?: string`, `timeout?: number` (default 2) | `number...` / `number?` | Look up computers hosting a protocol. |
| `run` | — | — | Start the background rednet listener (auto-started). |

### `parallel`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `waitForAny` | `...: function` | — | Run functions in parallel until any finishes. |
| `waitForAll` | `...: function(spawn)` | — | Run functions in parallel until all finish. Supports `spawn` for dynamic addition. |

### `window`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `create` | `parent: Redirect`, `x: number`, `y: number`, `width: number`, `height: number`, `visible?: boolean` (default true) | `Window` | Create a window within a parent terminal. |

**Window type** - extends `term.Redirect` with: `getLine(y)` returns `text, textColor, backgroundColor`, `setVisible(visible)`, `isVisible()`, `redraw()`, `restoreCursor()`, `getPosition()` returns `x, y`, `reposition(x, y, width?, height?, parent?)`.

### `io`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `stdin` | — | `Handle` | Standard input handle. |
| `stdout` | — | `Handle` | Standard output handle. |
| `stderr` | — | `Handle` | Standard error handle. |
| `close` | `file?: Handle` | — | Close a file handle. |
| `flush` | — | — | Flush the current output file. |
| `input` | `file?: Handle|string` | `Handle` | Get or set the current input file. |
| `lines` | `filename?: string`, `...` | `function` | Iterator over file lines. |
| `open` | `filename: string`, `mode?: string` (default "r") | `Handle?` / `nil, string` | Open a file. Modes: `r`, `w`, `a`, `r+`, `w+`; `b` suffix for binary. |
| `output` | `file?: Handle|string` | `Handle` | Get or set the current output file. |
| `read` | `...` | `string?` | Read from the current input file. |
| `type` | `obj: any` | `string?` | Check if value is a file handle ("file"/"closed file"). |
| `write` | `...` | — | Write to the current output file. |

**Handle type** - methods: `close()`, `flush()`, `lines(...)`, `read(...)` (formats: `l`, `L`, `a`), `seek(whence?: string, offset?: number)`, `setvbuf(mode, size?)` (no effect), `write(...)`.

### `keys`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `getName` | `code: number` | `string?` | Translate a key code to its name. |
**Key constants** (selected): `space=32`, `apostrophe=39`, `comma=44`, `minus=45`, `period=46`, `slash=47`, `zero`-`nine=48`-`57`, `semicolon=59`, `equals=61`, `a`-`z=65`-`90`, `leftBracket=91`, `backslash=92`, `rightBracket=93`, `grave=96`, `enter=257`, `tab=258`, `backspace=259`, `insert=260`, `delete=261`, `right=262`, `left=263`, `down=264`, `up=265`, `pageUp=266`, `pageDown=267`, `home=268`, `end=269`, `capsLock=280`, `scrollLock=281`, `numLock=282`, `printScreen=283`, `pause=284`, `f1`-`f25=290`-`313`, `numPad0`-`numPad9=320`-`329`, `numPadDecimal=330`, `numPadDivide=331`, `numPadMultiply=332`, `numPadSubtract=333`, `numPadAdd=334`, `numPadEnter=335`, `numPadEqual=336`, `leftShift=340`, `leftCtrl=341`, `leftAlt=342`, `leftSuper=343`, `rightShift=344`, `rightCtrl=345`, `rightAlt=346`, `menu=348`. Aliases: `return` = `enter`, `scollLock` = `scrollLock`.


### `disk`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `isPresent` | `name: string` | `boolean` | Check if a disk is in the drive. |
| `getLabel` | `name: string` | `string?` | Get the disk's label. |
| `setLabel` | `name: string`, `label: string?` | — | Set or clear the disk's label. |
| `hasData` | `name: string` | `boolean` | Check if disk provides a mount. |
| `getMountPath` | `name: string` | `string?` | Get the mount path. |
| `hasAudio` | `name: string` | `boolean` | Check if disk is a music record. |
| `getAudioTitle` | `name: string` | `string|false|nil` | Get the audio track title. |
| `playAudio` | `name: string` | — | Start playing the record. |
| `stopAudio` | `name: string` | — | Stop playing audio. |
| `eject` | `name: string` | — | Eject the disk. |
| `getID` | `name: string` | `string?` | Get the disk ID. |

### `peripheral`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `getNames` | — | `{string...}` | List all attached peripherals. |
| `isPresent` | `name: string` | `boolean` | Check if a peripheral is present. |
| `getType` | `peripheral: string|table` | `string...` | Get peripheral types. |
| `hasType` | `peripheral: string|table`, `type: string` | `boolean?` | Check if peripheral has a type. |
| `getMethods` | `name: string` | `{string...}?` | List methods for a peripheral. |
| `getName` | `peripheral: table` | `string` | Get a wrapped peripheral's name. |
| `call` | `name: string`, `method: string`, `...` | ... | Call a peripheral method. |
| `wrap` | `name: string` | `table?` | Wrap a peripheral into a table of methods. |
| `find` | `type: string`, `filter?: function(name, wrapped)` | `table...` | Find all peripherals of a type. |

### `pocket`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `equipBack` | — | `boolean, string?` | Equip an upgrade from the player's inventory. |
| `unequipBack` | — | `boolean, string?` | Remove the pocket computer's upgrade. |

---

## 3. Library Modules (`cc.*`)

### `cc.audio.dfpwm`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `make_encoder` | — | `function(pcm: {number...}):string` | Create a new PCM-to-DFPWM encoder. |
| `encode` | `input: {number...}` | `string` | Encode a complete audio buffer to DFPWM. |
| `make_decoder` | — | `function(dfpwm: string):{number...}` | Create a new DFPWM-to-PCM decoder. |
| `decode` | `input: string` | `{number...}` | Decode a complete DFPWM audio string. |

### `cc.base64`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `encode` | `str: string`, `alt_chars?: string` (default "+/") | `string` | Encode binary data to Base64. |
| `decode` | `str: string`, `alt_chars?: string` (default "+/") | `string` / `nil, string` | Decode Base64 data. |

### `cc.completion`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `choice` | `text: string`, `choices: {string...}`, `add_space?: boolean` | `{string...}` | Complete from a choice of strings. |
| `peripheral` | `text: string`, `add_space?: boolean` | `{string...}` | Complete a peripheral name. |
| `side` | `text: string`, `add_space?: boolean` | `{string...}` | Complete a side name. |
| `setting` | `text: string`, `add_space?: boolean` | `{string...}` | Complete a setting name. |
| `command` | `text: string`, `add_space?: boolean` | `{string...}` | Complete a Minecraft command name. |

### `cc.expect`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `expect` | `index: number`, `value: any`, `...: string` | `any` | Expect an argument to have a type. |
| `field` | `tbl: table`, `index: string`, `...: string` | `any` | Expect a table field to have a type. |
| `range` | `num: number`, `min?: number`, `max?: number` | `number` | Expect a number within a range. |

### `cc.image.nft`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `parse` | `image: string` | `table` | Parse an nft image from a string. |
| `load` | `path: string` | `table?` / `nil, string` | Load an nft image from a file. |
| `draw` | `image: table`, `x: number`, `y: number`, `target?: Redirect` | — | Draw an nft image. |

### `cc.pretty`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `empty` | — | `Doc` | An empty document. |
| `space` | — | `Doc` | A single space. |
| `line` | — | `Doc` | A line break (collapsible to empty). |
| `space_line` | — | `Doc` | A line break (collapsible to space). |
| `text` | `text: string`, `colour?: number` | `Doc` | Create a document from a string. |
| `concat` | `...: Doc|string` | `Doc` | Concatenate documents. Also supports `..`. |
| `nest` | `depth: number`, `doc: Doc` | `Doc` | Indent later lines. |
| `group` | `doc: Doc` | `Doc` | Display on one line if it fits. |
| `write` | `doc: Doc`, `ribbon_frac?: number` (default 0.6) | — | Display a document on the terminal. |
| `print` | `doc: Doc`, `ribbon_frac?: number` (default 0.6) | — | Display a document with trailing newline. |
| `render` | `doc: Doc`, `width?: number`, `ribbon_frac?: number` (default 0.6) | `string` | Render a document to a string. |
| `pretty` | `obj: any`, `options?: {function_args?: boolean, function_source?: boolean}` | `Doc` | Pretty-print an object. |
| `pretty_print` | `obj: any`, `options?: ...` | — | Pretty-print and print an object. |

**Doc type** — a document containing formatted text with multiple possible layouts. Supports `..` concatenation.

### `cc.require`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `make` | `env: table`, `dir: string` | `function, table` | Build a `require` function and `package` library. |

### `cc.shell.completion`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `file` | `shell: table`, `text: string` | `{string...}` | Complete a file name. |
| `dir` | `shell: table`, `text: string` | `{string...}` | Complete a directory name. |
| `dirOrFile` | `shell: table`, `text: string`, `previous: {string...}`, `add_space?: boolean` | `{string...}` | Complete a file or directory. |
| `program` | `shell: table`, `text: string` | `{string...}` | Complete a program name. |
| `programWithArgs` | `shell: table`, `text: string`, `previous: {string...}`, `starting: number` | `{string...}` | Complete program arguments. |
| `help` | — | `function` | Wrap `help.completeTopic` for `build`. |
| `choice` | — | `function` | Wrap `cc.completion.choice` for `build`. |
| `peripheral` | — | `function` | Wrap `cc.completion.peripheral` for `build`. |
| `side` | — | `function` | Wrap `cc.completion.side` for `build`. |
| `setting` | — | `function` | Wrap `cc.completion.setting` for `build`. |
| `command` | — | `function` | Wrap `cc.completion.command` for `build`. |
| `build` | `...: nil|function|{function, ...}` | `function` | Build a shell completion function. Supports `many` key for repeated args. |

### `cc.strings`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `wrap` | `text: string`, `width?: number` | `{string...}` | Wrap text to fit a width. |
| `ensure_width` | `line: string`, `width?: number` | `string` | Pad or truncate to a fixed width. |
| `split` | `str: string`, `deliminator: string`, `plain?: boolean` (default false), `limit?: number` | `{string...}` | Split a string by a delimiter. |

---

## 4. Peripherals

### `command`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `getCommand` | — | `string` | Get the command block's command. |
| `setCommand` | `command: string` | — | Set the command block's command. |
| `runCommand` | — | `boolean, string?` | Execute the command block. |

### `computer`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `turnOn` | — | — | Turn the computer on. |
| `shutdown` | — | — | Shut the computer down. |
| `reboot` | — | — | Reboot or turn on the computer. |
| `getID` | — | `number` | Get the computer's ID. |
| `isOn` | — | `boolean` | Check if the computer is on. |
| `getLabel` | — | `string?` | Get the computer's label. |

### `drive`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `isDiskPresent` | — | `boolean` | Check if a disk is inserted. |
| `getDiskLabel` | — | `string?` | Get the disk's label. |
| `setDiskLabel` | `label?: string` | — | Set or clear the disk's label. |
| `hasData` | — | `boolean` | Check if the disk has a mount. |
| `getMountPath` | — | `string?` | Get the mount path. |
| `hasAudio` | — | `boolean` | Check if the disk is a music record. |
| `getAudioTitle` | — | `string|false|nil` | Get the audio title. |
| `playAudio` | — | — | Play the record. |
| `stopAudio` | — | — | Stop playing audio. |
| `ejectDisk` | — | — | Eject the disk. |
| `getDiskID` | — | `number?` | Get the disk ID. |

### `modem`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `open` | `channel: number` | — | Open a channel (max 128). |
| `isOpen` | `channel: number` | `boolean` | Check if a channel is open. |
| `close` | `channel: number` | — | Close a channel. |
| `closeAll` | — | — | Close all channels. |
| `transmit` | `channel: number`, `replyChannel: number`, `payload: any` | — | Send a message on a channel. |
| `isWireless` | — | `boolean` | Check if this is a wireless modem. |
| `getNamesRemote` | — | `{string...}` | List remote peripherals (wired modems only). |
| `isPresentRemote` | `name: string` | `boolean` | Check if a remote peripheral exists (wired only). |
| `getTypeRemote` | `name: string` | `string...` | Get remote peripheral types (wired only). |
| `hasTypeRemote` | `name: string`, `type: string` | `boolean?` | Check if remote peripheral has a type (wired only). |
| `getMethodsRemote` | `name: string` | `{string...}?` | Get remote peripheral methods (wired only). |
| `callRemote` | `remoteName: string`, `method: string`, `...` | ... | Call a method on a remote peripheral (wired only). |
| `getNameLocal` | — | `string?` | Get this computer's wired network name (wired only). |

### `monitor`

Acts as a `term.Redirect` with additional methods. Exposes all `term.*` methods plus:

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `setTextScale` | `scale: number` | — | Set the monitor's text scale (0.5-5 multiples). |
| `getTextScale` | — | `number` | Get the monitor's text scale. |
| `write` | `text: string` | — | Write text at cursor position. |

Also inherits: `scroll`, `getCursorPos`, `setCursorPos`, `getCursorBlink`, `setCursorBlink`, `getSize`, `clear`, `clearLine`, `getTextColour`/`getTextColor`, `setTextColour`/`setTextColor`, `getBackgroundColour`/`getBackgroundColor`, `setBackgroundColour`/`setBackgroundColor`, `isColour`/`isColor`, `blit`, `setPaletteColour`/`setPaletteColor`, `getPaletteColour`/`getPaletteColor`.

### `printer`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `write` | `text: string` | — | Write text to the current page. |
| `getCursorPos` | — | `number, number` | Get cursor X, Y on the page. |
| `setCursorPos` | `x: number`, `y: number` | — | Set cursor position on the page. |
| `getPageSize` | — | `number, number` | Get the page width and height. |
| `newPage` | — | `boolean` | Start a new page. |
| `endPage` | — | `boolean` | Finalize and output the page. |
| `setPageTitle` | `title?: string` | — | Set or clear the page title. |
| `getInkLevel` | — | `number` | Get remaining ink. |
| `getPaperLevel` | — | `number` | Get remaining paper. |

### `redstone_relay`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `setOutput` | `side: string`, `on: boolean` | — | Turn redstone output on/off (strength 15). |
| `getOutput` | `side: string` | `boolean` | Get redstone output state. |
| `getInput` | `side: string` | `boolean` | Get redstone input state. |
| `setAnalogOutput` | `side: string`, `value: number` (0-15) | — | Set analog output strength. |
| `setAnalogueOutput` | `side: string`, `value: number` (0-15) | — | Alias for `setAnalogOutput`. |
| `getAnalogOutput` | `side: string` | `number` | Get analog output strength. |
| `getAnalogueOutput` | `side: string` | `number` | Alias for `getAnalogOutput`. |
| `getAnalogInput` | `side: string` | `number` | Get analog input strength. |
| `getAnalogueInput` | `side: string` | `number` | Alias for `getAnalogInput`. |
| `setBundledOutput` | `side: string`, `output: number` | — | Set bundled cable output. |
| `getBundledOutput` | `side: string` | `number` | Get bundled cable output. |
| `getBundledInput` | `side: string` | `number` | Get bundled cable input. |
| `testBundledInput` | `side: string`, `mask: number` | `boolean` | Test if colours in mask are set. |

### `speaker`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `playNote` | `instrument: string`, `volume?: number` (default 1.0), `pitch?: number` (0-24, default 12) | `boolean` | Play a noteblock note. Max 8 notes/tick. |
| `playSound` | `name: string`, `volume?: number` (default 1.0), `pitch?: number` (0.5-2.0, default 1.0) | `boolean` | Play a Minecraft sound. |
| `playAudio` | `audio: {number...}`, `volume?: number` | `boolean` | Stream PCM audio data. Max 128x1024 samples. |
| `stop` | — | — | Stop all audio playback. |

**Valid instruments:** `harp`, `basedrum`, `snare`, `hat`, `bass`, `flute`, `bell`, `guitar`, `chime`, `xylophone`, `iron_xylophone`, `cow_bell`, `didgeridoo`, `bit`, `banjo`, `pling`.

---

## 5. Generic Peripherals

### `energy_storage`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `getEnergy` | — | `number` | Get stored energy (FE). |
| `getEnergyCapacity` | — | `number` | Get maximum energy capacity. |

### `fluid_storage`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `tanks` | — | `{table?}` | Get all tanks with fluid info. |
| `pushFluid` | `toName: string`, `limit?: number`, `fluidName?: string` | `number` | Push fluid to a connected container. |
| `pullFluid` | `fromName: string`, `limit?: number`, `fluidName?: string` | `number` | Pull fluid from a connected container. |

### `inventory`

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `size` | — | `number` | Get number of slots. |
| `list` | — | `{table?}` | List all items (sparse, use `pairs`). |
| `getItemDetail` | `slot: number` | `table?` | Get detailed info about an item. |
| `getItemLimit` | `slot: number` | `number` | Get max stack size for a slot. |
| `pushItems` | `toName: string`, `fromSlot: number`, `limit?: number`, `toSlot?: number` | `number` | Push items to another inventory. |
| `pullItems` | `fromName: string`, `fromSlot: number`, `limit?: number`, `toSlot?: number` | `number` | Pull items from another inventory. |

---

## 6. Events

### `alarm`

Fired when an alarm set with `os.setAlarm` completes.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"alarm"`). |
| 2 | `number` | The alarm ID that finished. |

### `char`

Fired when a character is typed. Unlike `key`, accounts for multi-key input combinations.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"char"`). |
| 2 | `string` | The character pressed. |

### `computer_command`

Fired when the `/computercraft queue` command is run for the current command computer.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"computer_command"`). |
| 2 | `string...` | Arguments passed to the command. |

### `disk`

Fired when a disk is inserted into an adjacent or networked disk drive.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"disk"`). |
| 2 | `string` | Side of the disk drive. |

### `disk_eject`

Fired when a disk is removed from an adjacent or networked disk drive.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"disk_eject"`). |
| 2 | `string` | Side of the disk drive. |

### `file_transfer`

Fired when a user drags-and-drops files onto a computer.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"file_transfer"`). |
| 2 | `TransferredFiles` | Object with `getFiles()` method. |

**Types:** `TransferredFiles` - has `getFiles()` returning `{TransferredFile}`. `TransferredFile` inherits from binary file handle, with `getName()` returning the file name.

### `http_check`

Fired when a URL check (`http.checkURLAsync`) finishes.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"http_check"`). |
| 2 | `string` | URL checked. |
| 3 | `boolean` | Whether the check succeeded. |
| 4 | `string?` | Failure reason if failed. |

### `http_failure`

Fired when an HTTP request fails.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"http_failure"`). |
| 2 | `string` | URL requested. |
| 3 | `string` | Error description. |
| 4 | `Response?` | Response handle if connection succeeded but server indicated failure. |

### `http_success`

Fired when an HTTP request succeeds.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"http_success"`). |
| 2 | `string` | URL requested. |
| 3 | `Response` | The successful HTTP response handle. |

### `key`

Fired when a key is pressed. Returns a numerical key code (use `keys` constants).

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"key"`). |
| 2 | `number` | Key code. |
| 3 | `boolean` | Whether the key event was generated while holding the key. |

### `key_up`

Fired when a key is released.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"key_up"`). |
| 2 | `number` | Key code. |

### `modem_message`

Fired when a message is received on an open channel on any modem.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"modem_message"`). |
| 2 | `string` | Side of the receiving modem. |
| 3 | `number` | Channel the message was sent on. |
| 4 | `number` | Reply channel set by sender. |
| 5 | `any` | The received message. |
| 6 | `number?` | Distance in blocks, or `nil` for inter-dimensional messages. |

### `monitor_resize`

Fired when an adjacent or networked monitor is resized.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"monitor_resize"`). |
| 2 | `string` | Side or network ID of the monitor. |

### `monitor_touch`

Fired when an adjacent or networked Advanced Monitor is right-clicked.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"monitor_touch"`). |
| 2 | `string` | Side or network ID of the monitor. |
| 3 | `number` | X coordinate of the touch. |
| 4 | `number` | Y coordinate of the touch. |

### `mouse_click`

Fired when the terminal is clicked with a mouse (advanced computers only).

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"mouse_click"`). |
| 2 | `number` | Mouse button (1=left, 2=right, 3=middle). |
| 3 | `number` | X-coordinate of the click. |
| 4 | `number` | Y-coordinate of the click. |

### `mouse_drag`

Fired every time the mouse is moved while a mouse button is held.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"mouse_drag"`). |
| 2 | `number` | Mouse button being pressed. |
| 3 | `number` | X-coordinate of the mouse. |
| 4 | `number` | Y-coordinate of the mouse. |

### `mouse_scroll`

Fired when a mouse wheel is scrolled in the terminal.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"mouse_scroll"`). |
| 2 | `number` | Direction of scroll (-1=up, 1=down). |
| 3 | `number` | X-coordinate of the mouse. |
| 4 | `number` | Y-coordinate of the mouse. |

### `mouse_up`

Fired when a mouse button is released or a held mouse leaves the terminal.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"mouse_up"`). |
| 2 | `number` | Mouse button that was released. |
| 3 | `number` | X-coordinate of the mouse. |
| 4 | `number` | Y-coordinate of the mouse. |

### `paste`

Fired when text is pasted into the computer through Ctrl-V (Cmd-V on Mac).

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"paste"`). |
| 2 | `string` | The text that was pasted. |

### `peripheral`

Fired when a peripheral is attached on a side or to a modem.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"peripheral"`). |
| 2 | `string` | Side the peripheral was attached to. |

### `peripheral_detach`

Fired when a peripheral is detached from a side or from a modem.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"peripheral_detach"`). |
| 2 | `string` | Side the peripheral was detached from. |

### `rednet_message`

Fired when a message is sent over Rednet. Usually handled by `rednet.receive`.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"rednet_message"`). |
| 2 | `number` | The ID of the sending computer. |
| 3 | `any` | The message sent. |
| 4 | `string?` | The protocol of the message, if provided. |

### `redstone`

Fired whenever any redstone inputs on the computer or relay change.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"redstone"`). |

### `setting_changed`

Fired when a setting is modified with the `settings` API.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"setting_changed"`). |
| 2 | `string` | The name of the setting that was changed. |
| 3 | `any` | The value the setting was set to. |
| 4 | `any` | The previous value of the setting. |

### `speaker_audio_empty`

Fired when a speaker has space for more audio data after `playAudio`.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"speaker_audio_empty"`). |
| 2 | `string` | The name of the speaker which is available to play more audio. |

### `task_complete`

Fired when an asynchronous task completes. Usually handled by the calling function.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"task_complete"`). |
| 2 | `number` | The ID of the task that completed. |
| 3 | `boolean` | Whether the command succeeded. |
| 4 | `string?` | If failed, an error message. (Absent on success.) |
| 5 | `...` | Any parameters returned from the command. |

### `term_resize`

Fired when the main terminal is resized (e.g. tab bar shown/hidden, monitor redirect resized).

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"term_resize"`). |

### `terminate`

Fired when `Ctrl-T` is held down. Handled by `os.pullEvent` (not returned). `os.pullEventRaw` will return it.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"terminate"`). |

### `timer`

Fired when a timer started with `os.startTimer` completes.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"timer"`). |
| 2 | `number` | The ID of the timer that finished. |

### `turtle_inventory`

Fired when a turtle's inventory is changed.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"turtle_inventory"`). |

### `websocket_closed`

Fired when an open WebSocket connection is closed.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"websocket_closed"`). |
| 2 | `string` | URL of the WebSocket. |
| 3 | `string?` | Server-provided close reason (nil if abnormal). |
| 4 | `number?` | Connection close code (nil if abnormal). |

### `websocket_failure`

Fired when a WebSocket connection request fails.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"websocket_failure"`). |
| 2 | `string` | URL requested. |
| 3 | `string` | Error describing the failure. |

### `websocket_message`

Fired when a message is received on an open WebSocket connection.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"websocket_message"`). |
| 2 | `string` | URL of the WebSocket. |
| 3 | `string` | Contents of the message. |
| 4 | `boolean` | Whether this is a binary message. |

### `websocket_success`

Fired when a WebSocket connection request returns successfully.

| Position | Returns | Description |
|---|---|---|
| 1 | `string` | Event name (`"websocket_success"`). |
| 2 | `string` | URL of the site. |
| 3 | `Websocket` | The WebSocket connection handle. |

*Last updated: 2026-07-30*
