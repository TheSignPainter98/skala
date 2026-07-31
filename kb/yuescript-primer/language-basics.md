# YueScript Primer — Language Basics

## 1. Whitespace & Structure

### Whitespace-Significant
YueScript is whitespace-sensitive. Use **spaces** (tabs are allowed but treated as 4 spaces) to delimit blocks:

```yuescript
-- Block structures use indentation:
f = ->
  x = 1
  y = 2
  x + y  -- last line is implicitly returned
```

### Statement Separator
A line ending ends a statement. Use **`;`** for multiple statements on one line:

```yuescript
a = 1; b = 2; print a + b
```

### Multiline Chaining
Chain function calls by keeping the same indent level with `\` or `::` prefix:

```yuescript
Rx.Observable
  .fromRange 1, 8
  \filter (x) -> x % 2 == 0
  \concat Rx.Observable.of 'who do we appreciate'
  \map (value) -> value .. '!'
  \subscribe print
```

## 2. Comments

### Single Line
```yuescript
-- I am a comment
str = value -- trailing comment on same line
```

### Multi-Line
```yuescript
str = --[[
This is a multi-line comment.
It can contain anything.
]] str
```

### Inline Comments (between args)
```yuescript
func --[[port]] 3000, --[[ip]] "192.168.1.1"
```

## 3. Literal Types

All Lua primitive types are supported:
- **Numbers**: `42`, `3.14`, `0xEF`, `0B1010`, `1_000_000` (underscore separators)
- **Strings**: `'single'`, `"double"`, multiline strings with `|` prefix
- **Booleans**: `true`, `false`
- **Nil**: `nil`

### String Interpolation
Inside double-quoted strings, insert expressions with `#{...}`:

```yuescript
name = "Alice"
msg = "Hello, #{name}"  -- "Hello, Alice"
```

### YAML-Style Multiline Strings
```yuescript
str = |
  key: value
  list:
    - item1
    - #{expr}
```
- Preserves internal indentation
- Auto-strips common leading whitespace
- Safe escaping for quotes/backslashes

## 4. Variables & Scope

### Default = local
```yuescript
x = 10  -- local x
```

### Force local
```yuescript
local y = 20
```

### Force global
```yuescript
global Z
Z = 30
```

### Forward Declare
```yuescript
do
  local *       -- all variables become local
  x = -> f()  -- f is implicitly local
  f = -> 42
```

## 5. Operators

### Binary Operators
All Lua operators: `+`, `-`, `*`, `/`, `//`, `%`, `^`, `..`, `==`, `~=`, `<`, `>`, `<=`, `>=`, `and`, `or`, `not`

### Special Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `!=` | Alias for `~=` | `a != b` |
| `\` `::` | Chaining call | `tb\func!` or `tb::func!` iff `tb ~= nil` |
| `[] =` | Table append | `tab[] = val` |
| `...` | Table spread | `{...other}`, `[..., other]` |
| `#` | Reversed index | `tb[#]` = last item |
| `<>` / `:<` | Metatable shortcut | `<__index>: mt` |
| `?` | Existence check | `x?.field`, `x["key"]?` |
| `|>` | Pipe operator | `x |> f` = `f(x)` |
| `??` | Nil coalescing | `a ?? fallback` |
| `??=` | Nil-coalesce assign | `a ??= val` if `a == nil` |

### Chained Comparisons
```yuescript
1 < 2 <= 2  -- short-circuit, middle expr evaluated once
```

### Table Appending
```yuescript
items = {}
items[] = "new"
items[] = ...other_list  -- spread appends all elements
```

### Table Spreading
```yuescript
a = {1, 2, x: 1}
b = {3, 4, y: 2}
merge = {...a, ...b}  -- {1, 2, 3, 4, x: 1, y: 2}
list = {...a,}  -- only array part: {1, 2}
```

### Metatable Shortcut `<>`
```yuescript
-- Creation with metatable
a = <__add>: add_mt, value: 1

-- Metatable access
tb.<>  -- get metatable
tb.<__index> = {}  -- set metatable

-- Destructure metatable
{:__index, :__add} = some_object
```

### Existence Check `?`
```yuescript
x?.field          -- x and x.field are not nil
x["key"]?         -- value at key or nil
tab?.field?.sub   -- deeply nested nil-safe
with? obj         -- only enters if obj != nil
```

### Pipe Operator `|>`
```yuescript
"hello" |> print                       -- insert as first arg
2 |> _ + 1                             -- _ is placeholder
data |> map(_) |> filter(_ > 0) |> sort
```

### Nil Coalescing `??`
```yuescript
a = b ?? c ?? d  -- leftmost non-nil
func a ?? {}     -- default value
```
