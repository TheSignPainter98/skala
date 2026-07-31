# YueScript Quick Reference

## One-Minute Overview

YueScript is a dynamic language that **compiles to Lua**. It is a MoonScript dialect designed for writing expressive, highly concise code — ideal for game logic, server scripts, and domain-specific applications.

| Feature | Example | Description |
|---------|---------|-------------|
| Functions | `f = (x) -> x + 1` | Arrow syntax |
| Objects/Classes | `class Cat\n  new: => @sound = "meow"` | Native class system |
| Destructuring | `{a, b} = tbl` / `[x, ...rest] = list` | Extract values easily |
| Comprehensions | `[x * 2 for x in *list when x > 0]` | List & table comprehensions |
| Pipe | `data |> transform |> filter |> result` | Pipeline operator |
| Metatables | `tb = <__index>: mt, field: 1` | Metatable shortcuts |
| Try/Catch | `try\n  bad()\ncatch e\n  warn e` | Error handling |
| Macros | `macro FOO = -> '"constant"'` / `$FOO` | Compile-time code gen |
| Export/Import | `export a = 1` / `import x from "mod"` | Module system |

## File Convention
- Files use `.yue` extension
- Compiles to standard Lua
- Variables are **local by default** unless declared `global`
