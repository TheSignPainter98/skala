# CC: Tweaked Lua Environment Primer

Concise reference of all functions, types, peripherals, modules, generic peripherals, and events in CC: Tweaked's Lua environment. Source: [tweaked.cc](https://tweaked.cc/).

> **British English:** `colours` aliases `colors` (`grey`/`lightGrey`); `serialise`/`unserialise` alias `serialize`/`unserialize`.

---

## Table of Contents

1. [Global Environment (`_G`)](#1-global-environment-_g)
2. [Core Modules](#2-core-modules): `os` · `fs` · `term` · `textutils` · `colors`/`colours` · `redstone` · `paintutils` · `vector` · `turtle` · `commands` · `http` · `settings` · `help` · `shell` · `multishell` · `gps` · `rednet` · `parallel` · `window` · `io` · `keys` · `disk` · `peripheral` · `pocket`
3. [Library Modules (`cc.*`)](#3-library-modules-cc): `cc.audio.dfpwm` · `cc.base64` · `cc.completion` · `cc.expect` · `cc.image.nft` · `cc.pretty` · `cc.require` · `cc.shell.completion` · `cc.strings`
4. [Peripherals](#4-peripherals): `command` · `computer` · `drive` · `modem` · `monitor` · `printer` · `redstone_relay` · `speaker`
5. [Generic Peripherals](#5-generic-peripherals): `energy_storage` · `fluid_storage` · `inventory`
6. [Events](#6-events): All 30+ events (alarm through websocket_success)

---

## 1. Global Environment (`_G`)


| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `sleep` | `time? number` (default 0) | — | Pauses for `time` seconds; yields. |
| `write` | `text: string` | `number` | Writes without newline. |
| `print` | `...: any` | `number` | Prints with newline. |
| `printError` | `...: any` | — | Prints in red. |
| `read` | `replaceChar? string`, `history? table`, `completeFn? fn(string):{string...}`, `default? string` | `string` | Reads user input. |

**Constants:** `_HOST: string`, `_CC_DEFAULT_SETTINGS: string`.

---

## 2. Core Modules

### `os`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `pullEvent` | `filter? string` | `string, ...` | Waits for event. Stops on `terminate`. |
| `pullEventRaw` | `filter? string` | `string, ...` | No `terminate` handling. |
| `version` | — | `string` | CraftOS version. |
| `run` | `env: table`, `path: string`, `...` | `boolean` | Runs program. |
| `queueEvent` | `name: string`, `...` | — | Queue event. |
| `startTimer` | `time: number` | `number` | Starts timer for `time` seconds. |
| `setAlarm` | `time: number` | `number` | In-game time alarm; fires `alarm`. |
| `cancelTimer` | `token: number` | — | Cancel timer. |
| `cancelAlarm` | `token: number` | — | Cancel alarm. |
| `shutdown` | — | — | Shutdown. |
| `reboot` | — | — | Reboot. |
| `getComputerID` | — | `number` | Computer ID. |
| `getComputerLabel` | — | `string?` | Label or nil. |
| `setComputerLabel` | `label? string` | — | Set/clear label. |
| `clock` | — | `number` | Uptime in seconds. |
| `time` | `locale: "ingame"|"utc"|"local"` | `number` | Current time (0-1, 0=6AM in-game). |
| `day` | `locale: "ingame"|"utc"|"local"` | `number` | Day count. |
| `epoch` | `locale: "ingame"|"utc"|"local"` | `number` | Milliseconds since epoch. 1 real sec = 72000 in-game ms. |
| `date` | `format? string`, `time? number` | `string|table` | `"*t"` returns `{year, month, day, hour, min, sec, wday, yday, isdst}`. |

### `fs`

All paths absolute. `Handle`: `read(format?)`, `readAll()`, `readLine()`, `write(data)`, `writeLine(data)`, `flush()`, `close()`, `seek(whence?: "set"|"cur"|"end", offset?: number)`.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `complete` | `path: string`, `location: string`, `include_files? boolean` (true), `include_dirs? boolean` (true) | `{string...}` | Path completion. |
| `find` | `path: string` | `{string...}` | Wildcard search (`?`/`*`). |
| `isDriveRoot` | `path: string` | `boolean` | Is mount. |
| `list` | `path: string` | `{string...}` | Directory contents. |
| `combine` | `path: string`, `...: string` | `string` | Join paths. |
| `getName` | `path: string` | `string` | File name. |
| `getDir` | `path: string` | `string` | Parent dir. |
| `getSize` | `path: string` | `number` | Size in bytes. |
| `exists` | `path: string` | `boolean` | Exists. |
| `isDir` | `path: string` | `boolean` | Is dir. |
| `isReadOnly` | `path: string` | `boolean` | Read-only. |
| `makeDir` | `path: string` | — | Create dirs. |
| `move` | `path: string`, `dest: string` | — | Move. |
| `copy` | `path: string`, `dest: string` | — | Copy. |
| `delete` | `path: string` | — | Delete. |
| `open` | `path: string`, `mode: string` | `Handle?` | Modes: `r`,`w`,`a`,`r+`,`w+`; `b` for binary. |
| `getDrive` | `path: string` | `string` | Mount name. |
| `getFreeSpace` | `path: string` | `number` | Free space. |
| `getCapacity` | `path: string` | `number` | Capacity. |
| `attributes` | `path: string` | `table` | `{size, isDir, isReadOnly, created, modified}`. |

### `term`

`Redirect` type has same methods as `term` (British aliases: `isColour`/`isColor`).

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `nativePaletteColour` | `colour: number` | `r, g, b` | Default RGB (0-1). |
| `write` | `text: string` | — | Write at cursor. |
| `scroll` | `y: number` | — | Scroll (positive=up). |
| `getCursorPos` | — | `x, y` | Cursor position. |
| `setCursorPos` | `x: number`, `y: number` | — | Set cursor. |
| `getCursorBlink` | — | `boolean` | Blinking? |
| `setCursorBlink` | `blink: boolean` | — | Set blink. |
| `getSize` | — | `width, height` | Dimensions. |
| `clear` | — | — | Clear with bg colour. |
| `clearLine` | — | — | Clear line. |
| `getTextColour` | — | `number` | Text colour. |
| `setTextColour` | `colour: number` | — | Set text colour. |
| `getBackgroundColour` | — | `number` | Bg colour. |
| `setBackgroundColour` | `colour: number` | — | Set bg colour. |
| `isColour` | — | `boolean` | Supports colour. |
| `blit` | `text: string`, `textColour: string`, `backgroundColour: string` | — | Per-char hex colours. |
| `setPaletteColour` | `colour: number`, `r|rgb: number`, `g?: number`, `b?: number` | — | Set palette (channels 0-1). |
| `getPaletteColour` | `colour: number` | `r, g, b` | Get palette. |
| `redirect` | `target: Redirect` | `Redirect?` | Redirect output. |
| `current` | — | `Redirect` | Current terminal. |
| `native` | — | `Redirect` | Native terminal. |

### `textutils`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `slowWrite` | `text: string`, `rate? number` (20) | — | Char-by-char write. |
| `slowPrint` | `text: string`, `rate? number` (20) | — | Char-by-char print. |
| `formatTime` | `time: number`, `24h? boolean` | `string` | Format time. |
| `pagedPrint` | `text: string`, `free_lines? number` | `number` | Paged print. |
| `tabulate` | `...: {string...} \| number` | — | Structured tables. |
| `pagedTabulate` | `...: {string...} \| number` | — | Tabulate with paging. |
| `serialize` | `t: any`, `opts?: {compact?: boolean, allow_repetitions?: boolean}` | `string` | Lua serialisation. |
| `unserialize` | `s: string` | `any?` | Parse serialised. |
| `serializeJSON` | `t: any`, `opts?: {nbt_style?, unicode_strings?, allow_repetitions?}` | `string` | JSON serialise. |
| `unserializeJSON` | `s: string`, `opts?: {nbt_style?, parse_null?, parse_empty_array?}` | `any?` | JSON parse. |
| `urlEncode` | `str: string` | `string` | URL-encode. |
| `complete` | `text: string`, `env? table` | `{string...}` | Complete Lua expression. |

**Constants:** `empty_json_array`, `json_null`.

### `colors` / `colours`

`colours` is British alias.

| Constant | Val | Blit | Hex | RGB |
|---|---|---|---|---|
| white | 1 | 0 | #F0F0F0 | 240,240,240 |
| orange | 2 | 1 | #F2B233 | 242,178,51 |
| magenta | 4 | 2 | #E57FD8 | 229,127,216 |
| lightBlue | 8 | 3 | #99B2F2 | 153,178,242 |
| yellow | 16 | 4 | #DEDE6C | 222,222,108 |
| lime | 32 | 5 | #7FCC19 | 127,204,25 |
| pink | 64 | 6 | #F2B2CC | 242,178,204 |
| gray/grey | 128 | 7 | #4C4C4C | 76,76,76 |
| lightGray/lightGrey | 256 | 8 | #999999 | 153,153,153 |
| cyan | 512 | 9 | #4C99B2 | 76,153,178 |
| purple | 1024 | a | #B266E5 | 178,102,229 |
| blue | 2048 | b | #3366CC | 51,102,204 |
| brown | 4096 | c | #7F664C | 127,102,76 |
| green | 8192 | d | #57A64E | 87,166,78 |
| red | 16384 | e | #CC4C4C | 204,76,76 |
| black | 32768 | f | #111111 | 17,17,17 |

**Functions:** `combine(...number):number`, `subtract(colors:number, ...number):number`, `test(colors:number, color:number):boolean`, `packRGB(r,g,b):number`, `unpackRGB(rgb):r,g,b`, `toBlit(color):string`, `fromBlit(hex):number`. (`rgb8` deprecated.)

### `redstone`

Also `rs`. Sides: `"top"`, `"bottom"`, `"left"`, `"right"`, `"front"`, `"back"`.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `getSides` | — | `{string...}` | Six sides. |
| `setOutput` | `side: string`, `on: boolean` | — | Output on/off (strength 15). |
| `getOutput` | `side: string` | `boolean` | Output state. |
| `getInput` | `side: string` | `boolean` | Input state. |
| `setAnalogOutput` | `side: string`, `value: number` (0-15) | — | Analog output. |
| `getAnalogOutput` | `side: string` | `number` | Output strength. |
| `getAnalogInput` | `side: string` | `number` | Input strength. |
| `setBundledOutput` | `side: string`, `output: number` | — | Bundled output. |
| `getBundledOutput` | `side: string` | `number` | Bundled output. |
| `getBundledInput` | `side: string` | `number` | Bundled input. |
| `testBundledInput` | `side: string`, `mask: number` | `boolean` | Test colour mask. |

### `paintutils`

May change cursor/colour.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `parseImage` | `image: string` | `table` | Parse image. |
| `loadImage` | `path: string` | `table?` | Load image. |
| `drawPixel` | `x, y: number`, `colour?: number` | — | Draw pixel. |
| `drawLine` | `sx, sy, ex, ey: number`, `colour?: number` | — | Draw line. |
| `drawBox` | `sx, sy, ex, ey: number`, `colour?: number` | — | Unfilled box. |
| `drawFilledBox` | `sx, sy, ex, ey: number`, `colour?: number` | — | Filled box. |
| `drawImage` | `image: table`, `x, y: number` | — | Draw image. |

### `vector`

`v1+v2`, `v1-v2`, `v*n`, `v/n` operators. `new(x,y,z):Vector`. Methods: `add(o)`, `sub(o)`, `mul(f)`, `div(f)`, `unm()`, `dot(o):number`, `cross(o):Vector`, `length():number`, `normalize():Vector`, `round(t?:number):Vector`, `tostring():string`, `equals(o):boolean`.

### `turtle`

Movement: `forward()`, `back()`, `up()`, `down()`, `turnLeft()`, `turnRight()` -> `boolean, string?`

Block: `dig(side?)`, `digUp(side?)`, `digDown(side?)` -> `boolean, string?` (side: `"left"`,`"right"`)

Place: `place(text?)`, `placeUp(text?)`, `placeDown(text?)` -> `boolean, string?`

Attack: `attack(side?)`, `attackUp(side?)`, `attackDown(side?)` -> `boolean, string?`

Detect: `detect()`, `detectUp()`, `detectDown()` -> `boolean`

Compare: `compare()`, `compareUp()`, `compareDown()` -> `boolean`

Inspect: `inspect()`, `inspectUp()`, `inspectDown()` -> `boolean, table|string`

Inventory: `select(slot:number)`, `getItemCount(slot?:number):number`, `getItemSpace(slot?:number):number`, `compareTo(slot:number):boolean`, `transferTo(slot:number, count?:number):boolean`, `getSelectedSlot():number`, `getItemDetail(slot?:number, detailed?:boolean):table?`

Transfer: `drop(count?:number)`, `dropUp(count?:number)`, `dropDown(count?:number)`, `suck(count?:number)`, `suckUp(count?:number)`, `suckDown(count?:number)` -> `boolean, string?`

Fuel: `getFuelLevel():number|"unlimited"`, `getFuelLimit():number|"unlimited"`, `refuel(count?:number):boolean, string?`

Upgrades: `equipLeft():boolean, string?`, `equipRight():boolean, string?`, `getEquippedLeft():table?`, `getEquippedRight():table?`

Crafting: `craft(limit?:number):boolean, string?`

### `commands` (command computers only)

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `exec` | `command: string` | `boolean, {string...}, number?` | Sync execute. |
| `execAsync` | `command: string` | `number` | Async; fires `task_complete`. |
| `list` | `...: string` | `{string...}` | List commands. |
| `getDimension` | — | `string` | Dimension. |
| `getBlockPosition` | — | `x, y, z` | Position. |
| `getBlockInfos` | `minX,Y,Z, maxX,Y,Z: number`, `dim?: string` | `{table...}` | Region blocks (max 4096). |
| `getBlockInfo` | `x, y, z: number`, `dim?: string` | `table` | Block info. |
| `getEntities` | `selector: string` | `{table...}` | Entities. |

`commands.native` - raw API. `commands.async` - async wrappers.

### `http`

All accept table form: `{url, body?, headers?, binary?, method?, redirect?, timeout?}`. **Types:** `Response` extends `Handle`; `getResponseCode():number,string`, `getResponseHeaders():{string=string}`. `Websocket`: `receive(timeout?):string,boolean|nil,string`, `send(msg:string, binary?:boolean)`, `close()`, `getResponseHeaders():{string=string}`.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `get` | `url: string`, `headers? {string=string}`, `binary? boolean` | `Response?` / `nil, string, Response?` | Sync GET. |
| `post` | `url: string`, `body: string`, `headers?`, `binary?` | `Response?` / `nil, string, Response?` | Sync POST. |
| `request` | `url, body?, headers?, binary?` | — | Async; fires `http_success`/`http_failure`. |
| `checkURL` | `url: string` | `true` / `false, string` | Sync URL check. |
| `checkURLAsync` | `url: string` | `true` / `false, string` | Async; fires `http_check`. |
| `websocket` | `url: string`, `headers?` | `Websocket?` / `false, string` | Sync websocket. |
| `websocketAsync` | `url: string`, `headers?` | — | Async; fires `websocket_success`/`websocket_failure`. |

### `settings`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `define` | `name: string`, `options?: {description?, default?, type?}` | — | `type`: `number`,`string`,`boolean`,`table`. |
| `undefine` | `name: string` | — | Remove. |
| `set` | `name: string`, `value: any` | — | Set (not `nil`); call `save` to persist. |
| `get` | `name: string`, `default?: any` | `any` | Uses defined default if unset. |
| `getDetails` | `name: string` | `table` | `{description, default, type, value}`. |
| `unset` | `name: string` | — | Reset to default; fires `setting_changed`. |
| `clear` | — | — | Reset all. |
| `getNames` | — | `{string...}` | All names (sorted). |
| `load` | `path?: string` (`.settings`) | `boolean` | Load. |
| `save` | `path?: string` (`.settings`) | `boolean` | Save. |

### `help`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `path` | — | `string` | Colon-separated paths. |
| `setPath` | `newPath: string` | — | Set paths. |
| `lookup` | `topic: string` | `string?` | Find help file. |
| `topics` | — | `{string...}` | All topics. |
| `completeTopic` | `prefix: string` | `{string...}` | Complete topic. |

### `shell`

Not a "true" API; injected by the shell. `execute` passes args verbatim; `run` parses.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `execute` | `command: string`, `...: string` | `boolean` | Verbatim args. |
| `run` | `...: string` | `boolean` | Parsed args. |
| `exit` | — | — | Exit shell. |
| `dir` | — | `string` | Working dir. |
| `setDir` | `dir: string` | — | Set working dir. |
| `path` | — | `string` | Program paths (colon-separated). |
| `setPath` | `path: string` | — | Set paths. |
| `resolve` | `path: string` | `string` | To absolute. |
| `resolveProgram` | `command: string` | `string?` | Resolve program. |
| `programs` | `include_hidden?: boolean` | `{string...}` | List programs. |
| `complete` | `sLine: string` | `{string...}?` | Complete command. |
| `completeProgram` | `program: string` | `{string...}` | Complete program name. |
| `setCompletionFunction` | `program: string`, `complete: fn` | — | Set tab-completion. |
| `getCompletionInfo` | — | `{string={fnComplete=fn}}` | All completion funcs. |
| `getRunningProgram` | — | `string` | Running path. |
| `setAlias` | `command: string`, `program: string` | — | Add alias. |
| `clearAlias` | `command: string` | — | Remove alias. |
| `aliases` | — | `{string=string}` | Current aliases. |
| `openTab` | `...: string` | `number` | Open multishell tab. |
| `switchTab` | `id: number` | — | Switch tab. |

### `multishell`

IDs not constant over a program's run.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `getFocus` | — | `number` | Visible process index. |
| `setFocus` | `n: number` | `boolean` | Switch to process. |
| `getTitle` | `n: number` | `string?` | Process title. |
| `setTitle` | `n: number`, `title: string` | — | Set title. |
| `getCurrent` | — | `number` | Executing process. |
| `launch` | `env: table`, `path: string`, `...` | `number` | Start process. |
| `getCount` | — | `number` | Process count. |

### `gps`

| Const | Val |
|---|---
| `CHANNEL_GPS` | 65534 |

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `locate` | `timeout?: number` (2), `debug?: boolean` (false) | `x, y, z` / `nil` | Get position. |

### `rednet`

| Const | Val |
|---|---
| `CHANNEL_BROADCAST` | 65535 |
| `CHANNEL_REPEAT` | 65533 |
| `MAX_ID_CHANNELS` | 65500 |

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `open` | `modem: string` | — | Open modem. |
| `close` | `modem?: string` | — | Close modem. |
| `isOpen` | `modem?: string` | `boolean` | Is open. |
| `send` | `recipient: number`, `msg: any`, `protocol?: string` | `boolean` | Send to computer. |
| `broadcast` | `msg: any`, `protocol?: string` | — | Broadcast. |
| `receive` | `protocol_filter?: string`, `timeout?: number` | `sender, msg, protocol?` / `nil` | Wait for message. |
| `host` | `protocol: string`, `hostname: string` | — | Register host. |
| `unhost` | `protocol: string` | — | Unregister. |
| `lookup` | `protocol: string`, `hostname?: string`, `timeout?: number` (2) | `number...` / `number?` | Lookup hosts. |
| `run` | — | — | Background listener (auto-started). |

### `parallel`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `waitForAny` | `...: function` | — | Until any finishes. |
| `waitForAll` | `...: function(spawn)` | — | Until all finish. |

### `window`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `create` | `parent: Redirect`, `x: number`, `y: number`, `w: number`, `h: number`, `visible?: boolean` (true) | `Window` | Create window. |

**Window:** extends `term.Redirect` with `getLine(y):text,tc,bc`, `setVisible(v)`, `isVisible()`, `redraw()`, `restoreCursor()`, `getPosition():x,y`, `reposition(x,y,w?,h?,parent?)`.

### `io`

Emulates Lua's `io`. `Handle`: `close()`, `flush()`, `lines(...)`, `read(format?)`: `l`/`L`/`a`, `seek(whence?: "set"|"cur"|"end", offset?: number)`, `setvbuf(mode, size?)` (no effect), `write(...)`.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `stdin` | — | `Handle` | Stdin. |
| `stdout` | — | `Handle` | Stdout. |
| `stderr` | — | `Handle` | Stderr. |
| `close` | `file?: Handle` | — | Close handle. |
| `flush` | — | — | Flush output. |
| `input` | `file?: Handle|string` | `Handle` | Get/set input. |
| `lines` | `filename?: string`, `...` | `fn` | Line iterator. |
| `open` | `filename: string`, `mode?: string` (r) | `Handle?` / `nil, string` | Modes: `r`,`w`,`a`,`r+`,`w+`; `b` for binary. |
| `output` | `file?: Handle|string` | `Handle` | Get/set output. |
| `read` | `...` | `string?` | Read from input. |
| `type` | `obj: any` | `string?` | `"file"`/`"closed file"`/`nil`. |
| `write` | `...` | — | Write to output. |

### `keys`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `getName` | `code: number` | `string?` | Key code to name. |

**Constants:** `space=32`, `apostrophe=39`, `comma=44`, `minus=45`, `period=46`, `slash=47`, `0`-`9=48`-`57`, `semicolon=59`, `equals=61`, `a`-`z=65`-`90`, `leftBracket=91`, `backslash=92`, `rightBracket=93`, `grave=96`, `enter=257`, `tab=258`, `backspace=259`, `insert=260`, `delete=261`, `right=262`, `left=263`, `down=264`, `up=265`, `pageUp=266`, `pageDown=267`, `home=268`, `end=269`, `capsLock=280`, `scrollLock=281`, `numLock=282`, `printScreen=283`, `pause=284`, `f1`-`f25=290`-`313`, `numPad0`-`numPad9=320`-`329`, `numPadDecimal=330`, `numPadDivide=331`, `numPadMultiply=332`, `numPadSubtract=333`, `numPadAdd=334`, `numPadEnter=335`, `numPadEqual=336`, `leftShift=340`, `leftCtrl=341`, `leftAlt=342`, `leftSuper=343`, `rightShift=344`, `rightCtrl=345`, `rightAlt=346`, `menu=348`. Aliases: `return=enter`, `scollLock=scrollLock`.

### `disk`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `isPresent` | `name: string` | `boolean` | Disk in drive? |
| `getLabel` | `name: string` | `string?` | Disk label. |
| `setLabel` | `name: string`, `label: string?` | — | Set/clear label. |
| `hasData` | `name: string` | `boolean` | Has mount. |
| `getMountPath` | `name: string` | `string?` | Mount path. |
| `hasAudio` | `name: string` | `boolean` | Is music record. |
| `getAudioTitle` | `name: string` | `string|false|nil` | Audio title. |
| `playAudio` | `name: string` | — | Play record. |
| `stopAudio` | `name: string` | — | Stop audio. |
| `eject` | `name: string` | — | Eject disk. |
| `getID` | `name: string` | `string?` | Disk ID. |

### `peripheral`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `getNames` | — | `{string...}` | All peripherals. |
| `isPresent` | `name: string` | `boolean` | Is present. |
| `getType` | `peripheral: string|table` | `string...` | Peripheral types. |
| `hasType` | `peripheral: string|table`, `type: string` | `boolean?` | Has type. |
| `getMethods` | `name: string` | `{string...}?` | List methods. |
| `getName` | `peripheral: table` | `string` | Wrapped name. |
| `call` | `name: string`, `method: string`, `...` | ... | Call method. |
| `wrap` | `name: string` | `table?` | Wrap peripheral. |
| `find` | `type: string`, `filter?: fn(name, wrapped)` | `table...` | Find peripherals. |

### `pocket`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `equipBack` | — | `boolean, string?` | Equip upgrade. |
| `unequipBack` | — | `boolean, string?` | Remove upgrade. |

---

## 3. Library Modules (`cc.*`)

### `cc.audio.dfpwm`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `make_encoder` | — | `fn(pcm:{number...}):string` | New encoder. |
| `encode` | `input: {number...}` | `string` | Encode to DFPWM. |
| `make_decoder` | — | `fn(dfpwm:string):{number...}` | New decoder. |
| `decode` | `input: string` | `{number...}` | Decode DFPWM. |

### `cc.base64`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `encode` | `str: string`, `alt_chars?: string` (default "+/") | `string` | Encode to Base64. |
| `decode` | `str: string`, `alt_chars?: string` (default "+/") | `string` / `nil, string` | Decode Base64. |

### `cc.completion`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `choice` | `text: string`, `choices: {string...}`, `add_space?: boolean` | `{string...}` | Complete from choices. |
| `peripheral` | `text: string`, `add_space?: boolean` | `{string...}` | Complete peripheral name. |
| `side` | `text: string`, `add_space?: boolean` | `{string...}` | Complete side. |
| `setting` | `text: string`, `add_space?: boolean` | `{string...}` | Complete setting. |
| `command` | `text: string`, `add_space?: boolean` | `{string...}` | Complete command. |

### `cc.expect`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `expect` | `index: number`, `value: any`, `...: string` | `any` | Check argument type. |
| `field` | `tbl: table`, `index: string`, `...: string` | `any` | Check field type. |
| `range` | `num: number`, `min?: number`, `max?: number` | `number` | Check numeric range. |

### `cc.image.nft`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `parse` | `image: string` | `table` | Parse nft image. |
| `load` | `path: string` | `table?` / `nil, string` | Load from file. |
| `draw` | `image: table`, `x: number`, `y: number`, `target?: Redirect` | — | Draw image. |

### `cc.pretty`

`Doc` type: supports `..` concatenation.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `empty` | — | `Doc` | Empty doc. |
| `space` | — | `Doc` | Space. |
| `line` | — | `Doc` | Line break (collapsible to empty). |
| `space_line` | — | `Doc` | Line break (collapsible to space). |
| `text` | `text: string`, `colour?: number` | `Doc` | From string. |
| `concat` | `...: Doc|string` | `Doc` | Concatenate. |
| `nest` | `depth: number`, `doc: Doc` | `Doc` | Indent later lines. |
| `group` | `doc: Doc` | `Doc` | Single line if fits. |
| `write` | `doc: Doc`, `ribbon_frac?: number` (0.6) | — | Display on terminal. |
| `print` | `doc: Doc`, `ribbon_frac?: number` (0.6) | — | Display with newline. |
| `render` | `doc: Doc`, `width?: number`, `ribbon_frac?: number` (0.6) | `string` | Render to string. |
| `pretty` | `obj: any`, `opts?: {function_args?: boolean, function_source?: boolean}` | `Doc` | Pretty-print. |
| `pretty_print` | `obj: any`, `opts?: ...` | — | Pretty-print and print. |

### `cc.require`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `make` | `env: table`, `dir: string` | `fn, table` | Build `require` and `package`. |

### `cc.shell.completion`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `file` | `shell: table`, `text: string` | `{string...}` | Complete file name. |
| `dir` | `shell: table`, `text: string` | `{string...}` | Complete dir name. |
| `dirOrFile` | `shell: table`, `text: string`, `prev: {string...}`, `add_space?: boolean` | `{string...}` | Complete file or dir. |
| `program` | `shell: table`, `text: string` | `{string...}` | Complete program name. |
| `programWithArgs` | `shell: table`, `text: string`, `prev: {string...}`, `starting: number` | `{string...}` | Complete program args. |
| `help` | — | `fn` | Wrap `help.completeTopic` for `build`. |
| `choice` | — | `fn` | Wrap `cc.completion.choice` for `build`. |
| `peripheral` | — | `fn` | Wrap `cc.completion.peripheral` for `build`. |
| `side` | — | `fn` | Wrap `cc.completion.side` for `build`. |
| `setting` | — | `fn` | Wrap `cc.completion.setting` for `build`. |
| `command` | — | `fn` | Wrap `cc.completion.command` for `build`. |
| `build` | `...: nil|fn|{fn, ...}` | `fn` | Build shell completion. `many` key for repeats. |

### `cc.strings`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `wrap` | `text: string`, `width?: number` | `{string...}` | Wrap text. |
| `ensure_width` | `line: string`, `width?: number` | `string` | Pad or truncate. |
| `split` | `str: string`, `delim: string`, `plain?: boolean` (false), `limit?: number` | `{string...}` | Split string. |

---

## 4. Peripherals

### `command`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `getCommand` | — | `string` | Command block's command. |
| `setCommand` | `command: string` | — | Set command. |
| `runCommand` | — | `boolean, string?` | Execute. |

### `computer`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `turnOn` | — | — | Turn on. |
| `shutdown` | — | — | Shutdown. |
| `reboot` | — | — | Reboot/turn on. |
| `getID` | — | `number` | Computer ID. |
| `isOn` | — | `boolean` | Is on. |
| `getLabel` | — | `string?` | Label. |

### `drive`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `isDiskPresent` | — | `boolean` | Disk inserted? |
| `getDiskLabel` | — | `string?` | Disk label. |
| `setDiskLabel` | `label?: string` | — | Set/clear label. |
| `hasData` | — | `boolean` | Has mount. |
| `getMountPath` | — | `string?` | Mount path. |
| `hasAudio` | — | `boolean` | Is music record. |
| `getAudioTitle` | — | `string|false|nil` | Audio title. |
| `playAudio` | — | — | Play record. |
| `stopAudio` | — | — | Stop audio. |
| `ejectDisk` | — | — | Eject disk. |
| `getDiskID` | — | `number?` | Disk ID. |

### `modem`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `open` | `channel: number` | — | Open channel (max 128). |
| `isOpen` | `channel: number` | `boolean` | Channel open? |
| `close` | `channel: number` | — | Close channel. |
| `closeAll` | — | — | Close all. |
| `transmit` | `channel: number`, `replyChannel: number`, `payload: any` | — | Send message. |
| `isWireless` | — | `boolean` | Wireless modem. |
| `getNamesRemote` | — | `{string...}` | Remote peripherals (wired only). |
| `isPresentRemote` | `name: string` | `boolean` | Remote present (wired only). |
| `getTypeRemote` | `name: string` | `string...` | Remote types (wired only). |
| `hasTypeRemote` | `name: string`, `type: string` | `boolean?` | Has remote type (wired only). |
| `getMethodsRemote` | `name: string` | `{string...}?` | Remote methods (wired only). |
| `callRemote` | `remoteName: string`, `method: string`, `...` | ... | Call remote (wired only). |
| `getNameLocal` | — | `string?` | Network name (wired only). |

### `monitor`

Acts as `term.Redirect` with additional methods. Inherits all `term.*` methods.

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `setTextScale` | `scale: number` | — | Set text scale (0.5-5 multiples). |
| `getTextScale` | — | `number` | Text scale. |
| `write` | `text: string` | — | Write at cursor. |

**Inherits:** `scroll`, `getCursorPos`, `setCursorPos`, `getCursorBlink`, `setCursorBlink`, `getSize`, `clear`, `clearLine`, `getTextColour`/`getTextColor`, `setTextColour`/`setTextColor`, `getBackgroundColour`/`getBackgroundColor`, `setBackgroundColour`/`setBackgroundColor`, `isColour`/`isColor`, `blit`, `setPaletteColour`/`setPaletteColor`, `getPaletteColour`/`getPaletteColor`.

### `printer`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `write` | `text: string` | — | Write to page. |
| `getCursorPos` | — | `number, number` | Cursor X, Y. |
| `setCursorPos` | `x: number`, `y: number` | — | Set cursor. |
| `getPageSize` | — | `number, number` | Page width, height. |
| `newPage` | — | `boolean` | Start new page. |
| `endPage` | — | `boolean` | Finalize page. |
| `setPageTitle` | `title?: string` | — | Set/clear title. |
| `getInkLevel` | — | `number` | Remaining ink. |
| `getPaperLevel` | — | `number` | Remaining paper. |

### `redstone_relay`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `setOutput` | `side: string`, `on: boolean` | — | Output on/off (strength 15). |
| `getOutput` | `side: string` | `boolean` | Output state. |
| `getInput` | `side: string` | `boolean` | Input state. |
| `setAnalogOutput` | `side: string`, `value: number` (0-15) | — | Analog output. |
| `setAnalogueOutput` | `side: string`, `value: number` (0-15) | — | Alias. |
| `getAnalogOutput` | `side: string` | `number` | Output strength. |
| `getAnalogueOutput` | `side: string` | `number` | Alias. |
| `getAnalogInput` | `side: string` | `number` | Input strength. |
| `getAnalogueInput` | `side: string` | `number` | Alias. |
| `setBundledOutput` | `side: string`, `output: number` | — | Bundled output. |
| `getBundledOutput` | `side: string` | `number` | Bundled output. |
| `getBundledInput` | `side: string` | `number` | Bundled input. |
| `testBundledInput` | `side: string`, `mask: number` | `boolean` | Test colour mask. |

### `speaker`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `playNote` | `instrument: string`, `volume?: number` (1.0), `pitch?: number` (0-24, 12) | `boolean` | Noteblock note. Max 8/tick. |
| `playSound` | `name: string`, `volume?: number` (1.0), `pitch?: number` (0.5-2.0, 1.0) | `boolean` | Minecraft sound. |
| `playAudio` | `audio: {number...}`, `volume?: number` | `boolean` | PCM audio. Max 128x1024 samples. |
| `stop` | — | — | Stop all audio. |

**Instruments:** `harp`, `basedrum`, `snare`, `hat`, `bass`, `flute`, `bell`, `guitar`, `chime`, `xylophone`, `iron_xylophone`, `cow_bell`, `didgeridoo`, `bit`, `banjo`, `pling`.

---

## 5. Generic Peripherals

### `energy_storage`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `getEnergy` | — | `number` | Stored energy (FE). |
| `getEnergyCapacity` | — | `number` | Max capacity. |

### `fluid_storage`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `tanks` | — | `{table?}` | All tanks (sparse, use `pairs`). |
| `pushFluid` | `toName: string`, `limit?: number`, `fluidName?: string` | `number` | Push fluid. |
| `pullFluid` | `fromName: string`, `limit?: number`, `fluidName?: string` | `number` | Pull fluid. |

### `inventory`

| Function | Parameters | Returns | Notes |
|---|---|---|---|
| `size` | — | `number` | Number of slots. |
| `list` | — | `{table?}` | All items (sparse, use `pairs`). Each: `{name, count, nbt?}`. |
| `getItemDetail` | `slot: number` | `table?` | Item details. |
| `getItemLimit` | `slot: number` | `number` | Max stack size. |
| `pushItems` | `toName: string`, `fromSlot: number`, `limit?: number`, `toSlot?: number` | `number` | Push items. |
| `pullItems` | `fromName: string`, `fromSlot: number`, `limit?: number`, `toSlot?: number` | `number` | Pull items. |

---

## 6. Events

### `alarm`

Fired when `os.setAlarm` completes.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"alarm"` |
| 2 | `number` | Alarm ID. |

### `char`

Fired when a character is typed (not a key press).

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"char"` |
| 2 | `string` | Character. |

### `computer_command`

Fired when `/computercraft queue` is run for current command computer.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"computer_command"` |
| 2 | `string...` | Arguments. |

### `disk`

Fired when a disk is inserted into a disk drive.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"disk"` |
| 2 | `string` | Side of drive. |

### `disk_eject`

Fired when a disk is removed from a disk drive.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"disk_eject"` |
| 2 | `string` | Side of drive. |

### `file_transfer`

Fired when files are dragged onto a computer.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"file_transfer"` |
| 2 | `TransferredFiles` | Has `getFiles():{TransferredFile}`. `TransferredFile` has `getName():string` and inherits file handle methods. |

### `http_check`

Fired when `http.checkURLAsync` finishes.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"http_check"` |
| 2 | `string` | URL. |
| 3 | `boolean` | Success. |
| 4 | `string?` | Failure reason. |

### `http_failure`

Fired when an HTTP request fails.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"http_failure"` |
| 2 | `string` | URL. |
| 3 | `string` | Error. |
| 4 | `Response?` | Response if connection succeeded but server failed. |

### `http_success`

Fired when an HTTP request succeeds.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"http_success"` |
| 2 | `string` | URL. |
| 3 | `Response` | Response handle. |

### `key`

Fired when a key is pressed (returns key codes; use `keys` constants).

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"key"` |
| 2 | `number` | Key code. |
| 3 | `boolean` | Held (true) vs pressed (false). |

### `key_up`

Fired when a key is released.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"key_up"` |
| 2 | `number` | Key code. |

### `modem_message`

Fired when a message is received on an open channel on any modem.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"modem_message"` |
| 2 | `string` | Receiving modem side. |
| 3 | `number` | Channel. |
| 4 | `number` | Reply channel. |
| 5 | `any` | Message. |
| 6 | `number?` | Distance in blocks, or nil for interdimensional. |

### `monitor_resize`

Fired when a monitor is resized.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"monitor_resize"` |
| 2 | `string` | Side or network ID. |

### `monitor_touch`

Fired when an Advanced Monitor is right-clicked.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"monitor_touch"` |
| 2 | `string` | Side or network ID. |
| 3 | `number` | X coordinate. |
| 4 | `number` | Y coordinate. |

### `mouse_click`

Fired when the terminal is clicked (advanced computers only). Buttons: 1=left, 2=right, 3=middle.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"mouse_click"` |
| 2 | `number` | Button. |
| 3 | `number` | X. |
| 4 | `number` | Y. |

### `mouse_drag`

Fired when mouse moves while a button is held.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"mouse_drag"` |
| 2 | `number` | Button. |
| 3 | `number` | X. |
| 4 | `number` | Y. |

### `mouse_scroll`

Fired when mouse wheel is scrolled. (-1=up, 1=down)

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"mouse_scroll"` |
| 2 | `number` | Direction (-1/1). |
| 3 | `number` | X. |
| 4 | `number` | Y. |

### `mouse_up`

Fired when a mouse button is released.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"mouse_up"` |
| 2 | `number` | Button. |
| 3 | `number` | X. |
| 4 | `number` | Y. |

### `paste`

Fired when text is pasted (Ctrl-V/Cmd-V).

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"paste"` |
| 2 | `string` | Pasted text. |

### `peripheral`

Fired when a peripheral is attached.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"peripheral"` |
| 2 | `string` | Side attached. |

### `peripheral_detach`

Fired when a peripheral is detached.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"peripheral_detach"` |
| 2 | `string` | Side detached. |

### `rednet_message`

Fired when a Rednet message is sent. Usually handled by `rednet.receive`.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"rednet_message"` |
| 2 | `number` | Sender ID. |
| 3 | `any` | Message. |
| 4 | `string?` | Protocol. |

### `redstone`

Fired when redstone inputs change.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"redstone"` |

### `setting_changed`

Fired when a setting is modified.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"setting_changed"` |
| 2 | `string` | Setting name. |
| 3 | `any` | New value. |
| 4 | `any` | Previous value. |

### `speaker_audio_empty`

Fired when a speaker has space for more audio.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"speaker_audio_empty"` |
| 2 | `string` | Speaker name. |

### `task_complete`

Fired when an async task completes.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"task_complete"` |
| 2 | `number` | Task ID. |
| 3 | `boolean` | Success. |
| 4 | `string?` | Error (on failure). |
| 5 | `...` | Return values. |

### `term_resize`

Fired when the terminal is resized.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"term_resize"` |

### `terminate`

Fired when Ctrl-T is held. Handled by `pullEvent` (not returned); `pullEventRaw` will return it.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"terminate"` |

### `timer`

Fired when `os.startTimer` completes.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"timer"` |
| 2 | `number` | Timer ID. |

### `turtle_inventory`

Fired when a turtle's inventory changes.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"turtle_inventory"` |

### `websocket_closed`

Fired when a WebSocket closes.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"websocket_closed"` |
| 2 | `string` | URL. |
| 3 | `string?` | Close reason (nil if abnormal). |
| 4 | `number?` | Close code (nil if abnormal). |

### `websocket_failure`

Fired when a WebSocket request fails.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"websocket_failure"` |
| 2 | `string` | URL. |
| 3 | `string` | Error. |

### `websocket_message`

Fired when a message is received on a WebSocket.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"websocket_message"` |
| 2 | `string` | URL. |
| 3 | `string` | Message. |
| 4 | `boolean` | Binary? |

### `websocket_success`

Fired when a WebSocket connects.

| Pos | Returns | Notes |
|---|---|---|
| 1 | `string` | `"websocket_success"` |
| 2 | `string` | URL. |
| 3 | `Websocket` | Connection handle. |
