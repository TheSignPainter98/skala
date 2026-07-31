# YueScript Primer — Advanced Features

## 1. Macros

### Define Macro
```yuescript
macro NAME = (args) ->
  -- return string (Yue code) or {code: ..., type: "yue"|"lua"|"text"}
  "generated code"

macro config = (debugging) ->
  global debugMode = debugging == "true"
  ""
```

### Invoke Macro
```yuescript
$NAME arg1, arg2   -- inline expansion
$NAME              -- no args
```

### Macro with Raw Lua
```yuescript
macro lua_only = (code) ->
  {code, type: "lua"}

$lua_only {==[
  -- raw Lua that won't be touched
  if x then return 1 end
]==]
```

### Macro Annotations (on next statement)
```yuescript
$[MacroName]
statement_here  -- gets statement AST as arg

$[MacroName arg]
class MyClass
  ...
```

### Macro Export
```yuescript
-- module.yue
export macro my_macro = (...) -> "result"

-- consumer.yue
import "module" as {$}
value = $my_macro arg
```

### Argument AST Type Checking
```yuescript
macro validate = (num `Num, str `String) ->
  "code..."

$validate 42, "hello"
```

### Builtin Macros
```yuescript
$FILE  -- current module name (string)
$LINE  -- current line number (integer)
```

## 2. Line Decorators
One-liner forms for loop/if:
```yuescript
print "hello" if condition
process item for item in *items
update! while game\running!
```

## 3. Do Expression
```yuescript
value = do
  expr1
  expr2
  result  -- implicit return

status, data = do
  if ok
    break "success", get_data!
  break "error", "fail"
```

## 4. Try / Catch
```yuescript
try
  risky_call!
catch e
  print yue.traceback e

success, result = try
  risky_call!
catch e
  log e

success, result = try risky_call!

try? risky_call!          -- return nil instead of error
success, result = try? risky_call!
```

### Try as expression in if
```yuescript
if success, result := try risky!
catch e
  handle_error e
```

## 5. The Using Clause
Prevents destructive reassignment to closed variables:
```yuescript
f = (args using outer_var) ->
  outer_var = new  -- creates new local, not modifying outer
```

```yuescript
f = (x using nil) ->
  -- no closed variables can be reassigned
```

## 6. Global Import
```yuescript
do
  import global
  print "hello"  -- print becomes local const to global
  FLAG = 123     -- if FLAG was globally declared, it is writable
```

## 7. Attributes (const / close)
```yuescript
const a = 123    -- const (Luajit/Lua 5.4 compatible)
close _ = <close>: -> cleanup!
```
