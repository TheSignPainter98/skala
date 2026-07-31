# YueScript Primer — Index

This directory contains comprehensive documentation for the YueScript language. YueScript compiles to Lua and extends Lua with concise, expressive syntax.

## Document Structure

| File | Topic |
|---|---|---|
| `quick-reference.md` | One-page reference |
| `language-basics.md` | Whitespace, comments, literals, variables, operators |
| `assignment.md` | Assignment, destructuring, if-assignment, using clause |
| `control-flow.md` | Conditionals, loops, switch, continue, goto |
| `functions.md` | Function definitions, parameters, backcalls, stubs |
| `data-structures.md` | Arrays, tables, comprehensions, slicing |
| `objects.md` | Classes, inheritance, with block, @@ |
| `advanced.md` | Macros, try/catch, do expression, line decorators |
| `modules.md` | Import/export, module system |

## Quick Patterns

### Function
```yuescript
add = (a, b) -> a + b
```

### Class
```yuescript
class Dog
  new: (@name) =>
  bark: => "Woof!"

dog = Dog "Buddy"
dog\bark!
```

### Destructuring
```yuescript
{a, :b} = tbl
[x, ...rest] = list
```

### Comprehension
```yuescript
[result for item in *items when item > 0]
```

### Pipe
```yuescript
data |> map(f) |> reduce(g)
```

### Metatable
```yuescript
obj = <__index>: mt, field: 1
obj.<__index>  -- access metatable
```

### Existence
```yuescript
x?.field?.sub
```

### Nil Coalescing
```yuescript
val = fallback ?? default
```

### Class inheritance
```yuescript
class Cat extends Pet
  new: =>
    super  -- call parent constructor
    @sound = "meow"
```

### Macro
```yuescript
macro CONST = -> '"static value"'
x = $CONST
