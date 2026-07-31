# YueScript Language Primer

## Overview
YueScript is a dynamic, expressive language that **compiles to Lua**. It is a MoonScript dialect designed for writing highly concise code — ideal for game logic, domain-specific applications, and server scripts in Lua-embedded environments.

- **File extension:** `.yue`
- **Target output:** Standard Lua code
- **Variable scope:** `local` by default unless declared `global`

---

## 1. Language Basics

### 1.1 Whitespace & Structure
YueScript is whitespace-sensitive. Use **spaces** (tabs are allowed but treated as 4 spaces) to delimit blocks.

```yuescript
f = ->
  x = 1
  x + 2  -- implicit return
```

**Statement Separator:** A line break ends a statement. Use **`;`** for multiple statements on one line:
```yuescript
a = 1; b = 2; print a + b
```

**Multiline Chaining:** Chain method/function calls by keeping the same indent level with `\` or `::` prefix:
```yuescript
Rx.Observable
  .fromRange 1, 8
  \filter (x) -> x % 2 == 0
  \concat Rx.Observable.of "appreciation"
  \subscribe print
```

### 1.2 Comments
- **Single-line:** `-- comment` (can trail code)
- **Multi-line:** `--[[ comment ]]`
- **Inline between args:** `func --[[port]] 3000`

### 1.3 Literals
All Lua primitives are supported. Yue adds underscore separators and YAML-style strings.

- **Numbers:** `42`, `3.14`, `0xEF`, `0B1010`, `1_000_000`
- **Strings:** `''`, `""`. Allows unescaped line breaks inside.
- **String Interpolation:** `msg = "Hello #{name}"` (double quotes only)
- **YAML Multiline Strings:**
  ```yuescript
  str = |
    key: value
    list:
      - item1
  ```
  Preserves internal whitespace, auto-strips the leading prefix, and safely escapes quotes.

### 1.4 Variables & Scope
Variables are `local` by default.
```yuescript
x = 10          -- local by default
local y = 10    -- explicit local
global z = 10   -- explicit global
```

**Forward Declaration:**
```yuescript
do
  local *       -- future variables are local
  local ^       -- future uppercase variables are local
  global *      -- future variables are global
  f = -> g()    -- g is implicitly local
  g = -> 42
end
```

### 1.5 Operators
All Lua binary/unary operators are available (`+`, `-`, `*`, `/`, `^`, `..`, `==`, `~=`, etc.).

**Special Operators:**
| Operator | Meaning | Example |
|----------|---------|---------|
| `!=` | Not equal (`~=` alias) | `a != b` |
| `\` or `::` | Optional chaining call | `tb\func!` if `tb` exists |
| `[] =` | Table append | `tab[] = val` |
| `...` | Spread | `{...a}`, `[...,a,]` |
| `#` | Reversed index | `tab[#]` = last item |
| `<>` | Metatable shortcut | `<__index>: mt` |
| `?` | Existence check | `x?.field` |
| `|>` | Pipe | `x \|> f` = `f(x)` |
| `??` | Nil coalescing | `a ?? fallback` |
| `??=` | Nil coalesce assign | `a ??= val` |

**Chained Comparisons:** `1 < 2 <= 2` (evaluates middle once, short-circuits)
**In Expression:** `if x in [1, 2, 3]` (membership check)
**Table Spreading:** `merge = {...a, ...b}` (braces merge arrays+hashes; brackets `[...,a,]` merge only arrays)

---

## 2. Assignment & Destructuring

### 2.1 Basic & Chaining
```yuescript
x += 1            -- perform update
arg or= "default" -- only if arg is nil
a = b = c = 0   -- chaining assignment
```

### 2.2 Destructuring
Swap table/array literal to the left-hand side to unpack values.
```yuescript
thing = [1, 2, 3]
[a, b, ...rest] = thing
obj = {hello: "world"}
{hello: hello} = obj
:day = obj        -- shorthand for {day: val}
```
- **Nested:** `{numbers: [first, second], props: {color: c}} = obj`
- **Defaults:** `{:name = "anon", :job = "u"} = person`
- **Range:** `[first, ...bulk, last] = list`
- **In For:** `for [a, b] in *tuples`

### 2.3 If Assignment (Walrus Operator)
Assign inside a condition; variable is scoped to that block.
```yuescript
if user := db.find "moon"
  print user.name
elseif val := os.getenv "X"
  print val
```

### 2.4 Varargs Assignment
Unpack multiple return values into `...`:
```yuescript
ok, ... = fn()
cnt = select '#, ...
```

### 2.5 Using Clause (Prevent Destructive Assignment)
Prevent accidentally overwriting outer scope variables:
```yuescript
f = (args using outer_var) ->
  outer_var = 10  -- creates new local, outer untouched
f = (x using nil) ->
  -- no closed vars can be modified
```

---

## 3. Control Flow

### 3.1 Conditionals
```yuescript
if cond
  body
elseif cond2
  body2
else
  body3
```
- **Inline:** `x if cond`, `x unless cond`
- **Expression:** `val = if cond then "yes" else "no"`
- **`in` expression:** `if x in list`
- **Switch:**
  ```yuescript
  switch val
    when "a"         -- single
    when "b", "c"    -- multiple
      print "b or c"
    else
      print "else"
  ```
- **Switch Table Matching:** `when :x, :y`, `when {width}`, `when [1, b, 3]`
- **Switch as expression** or inline (`when 1 then "one"`)

### 3.2 Loops
- **Numeric:** `for i = 1, 10, 2 do body`
- **Generic:** `for k, v in pairs tbl`
- **Sliced:** `for item in *items[2, 4]`
- **As expression (accumulator):**
  ```yuescript
  list = for i = 1, 5
    i * 2
  first_found = for n in *nums
    break n if n > 10
  ```

### 3.3 While / Until / Repeat
```yuescript
while cond      -- until cond false
  body
until cond      -- repeats until cond true
  body
repeat
  body
until cond
```

### 3.4 Continue & Goto
- **Continue:** `continue if odd` (skips remaining body)
- **Goto:** `::label ::` / `goto label if cond` (requires Lua 5.2+). Labels scoped locally; cannot jump into inner blocks.

---

## 4. Functions

### 4.1 Definitions & Call
```yuescript
f = ->                  -- no args, no body
f = -> expr             -- implicit return
f = (a, b) -> a + b    -- with args
f!                      -- call (empty args)
f arg1, arg2            -- parenless call
f(arg)                  -- parenthesized
```
- **Fat Arrow `=>`** auto-binds `@` (self).
- **Defaults:** `f = (a = 1, b = a + 1) -> ...`
- **Parameter Destructuring:** `f = (:a, b: b1) -> ...`
- **Prefixed Return:** `f = (list): nil -> loop ...`
- **Named Varargs:** `f = (...t) -> ... t.n contains arg count`

### 4.2 Special Syntax
- **Backcalls (`<-`):** Unnest deeply nested callbacks.
  `x <- f; body` where `x` is the async result.
- **Function Stubs:** `my_object\write` bundles object and method.
- **Implict Objects:**
  ```yuescript
  func
    * 1
    * 2
  ```

---

## 5. Data Structures

### 5.1 Arrays & Tables
- **Array:** `[1, 2, 3]`
- **Table:** `{a: 1, b: 2}` or braceless `key: val`
- **Spread in Tables:** `a = {...other}` (both parts), `list = {...a,}` (array only)
- **Implicit Objects (indent-based):**
  ```yuescript
  obj =
    name: "Bob"
    items:
      - "a"
  ```

### 5.2 Comprehensions
- **List:** `[item * 2 for item in *items when item > 0]`
- **Table:** `{k, v for k, v in pairs tbl}`
- **Nested Loops:** `[x + y for x in *a for y in *b]`
- **Numeric:** `[i for i = 1, 100 when i % 2 == 0]`
- **Flatten:** `[...sub for item in *items]`

### 5.3 Slicing
```yuescript
slice = items[2, 4]      -- items 2 to 4
last  = items[-4, -1]    -- last 4 items
every = items[1,, 2]     -- step by 2
rev   = items[-1, 1, -1] -- reversed
```

---

## 6. Objects & OOP

### 6.1 Classes
```yuescript
class Dog
  new: (@name) =>        -- parameter promotion
  bark: => "Woof!"

dog = Dog "Rex"
dog\bark!
```
- `@` accesses instance fields. `@@` accesses class fields.
- **Inheritance:** `class Cat extends Dog`
- **Super:** `super args` or `super.method`
- **Inherited Hook:** `@__inherited: (child) => ...`
- **Class Mixing:** `class MyClass using MixinTable`
- **Anonymous:** `cls = class extends Base`

### 6.2 The `with` Block
Avoids repeating object names:
```yuescript
with obj
  .field = 1
  \method!
  print .field

with? maybe_nil  -- nil-safe
  \do!
```

### 6.3 Type & Metadata
Instances carry `obj.__class`. Classes have `ClassName.__name`, `.__base`, `.__parent`.

---

## 7. Module System

### 7.1 Import
```yuescript
import a, b from "mod"
import "mod" as :X, :Y       -- extract multiple
import "mod" as alias         -- full require
import "mod" as {a: X, b}     -- rename/destructure
from "mod" import a, b        -- Python style
do import global              -- auto-import all unassigned globals as const
```
### 7.2 Export
```yuescript
export x = 1, class MyClass
export my_func = -> ...
export default class_val
export.key = tbl              -- indexed
export a, b                   -- positional array export
```

---

## 8. Advanced Features

### 8.1 Macros
Compile-time code generation.
```yuescript
macro NAME = (args) ->
  "generated string"          -- returns Yue code
  -- or {code: "...", type: "lua"|"text"} for raw Lua

$NAME arg                     -- invoke inline
```
- **Annotation:** `$[Macro]` applies macro to the *next* statement's AST.
- **Export/Import macros:** `export macro ...` / `import "mod" as {$}`.
- **AST checking:** `macro fn(a \`Num) ->`
- **Builtins:** `$FILE` (string), `$LINE` (integer).

### 8.2 Line Decorators
One-liner control flow:
```yuescript
print "hi" if cond
process item for item in *items
f! while is_running
```

### 8.3 Do Expression
Multi-line scoped expression returning the last line:
```yuescript
val = do
  a = 1
  a + 2

status, data = do
  if ok
    break "ok", val
  break "fail", err
```

### 8.4 Try / Catch
```yuescript
try
  risky_call!
catch e
  print yue.traceback e

success, res = try risky_call!
val = try? risky_call!        -- returns nil on error, not error object
```
Can be combined with `if success, res := try ...`

### 8.5 Attributes
```yuescript
const a = 123    -- const (Luajit/Lua 5.4 comp)
close _ = <close>: -> cleanup!
```
