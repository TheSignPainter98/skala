# YueScript Primer — Functions

## 1. Function Definitions

### Basic Arrow Function
```yuescript
f = ->              -- no params, no body
f = -> expr         -- one-line body (implicit return)
f = ->
  body              -- multi-line body (implicit return)
```

### With Parameters
```yuescript
sum = (a, b) -> a + b
```

### Invocation
```yuescript
f!                  -- no-arg call (preferred)
f()                 -- also works
f arg1, arg2        -- paren-less call
f(arg1, arg2)       -- parenthesized (no space before `(`)
```

### Implicit Return
Last expression is returned:
```yuescript
double = (x) -> x * 2
```

### Explicit Return
```yuescript
compute = (x) -> return x * 2
```

### Multiple Returns
```yuescript
mystery = (x, y) -> x + y, x - y
a, b = mystery 10, 20
```

## 2. Fat Arrow `=>` (Method Functions)

Auto-binds `@` (self):
```yuescript
add = (right) => @value + right.value
```

## 3. Parameter Defaults
```yuescript
f = (name = "default", count = 10) ->
  print name, count
```
Defaults are evaluated in order; earlier params available to later ones.

## 4. Parameter Destructuring
```yuescript
f1 = (:a, :b, c) ->   -- destructure first param
f2 = ({a: a1 = 123}, c) ->  -- destructure with default
f3 = (:a, b: b1) ->   -- mix of : prefix and key: value
```

## 5. Prefixed Return Expression
Default return before the arrow:
```yuescript
findFirst = (list): nil ->
  for item in *list
    if item > 0
      return item
```
Equivalent to appending `nil` at the end (or any default return).

## 6. Named Varargs
```yuescript
f = (...t) ->   -- t is a table with values + t.n = arg count
  for i = 1, t.n
    print t[i]
```

## 7. Backcalls (Unnest Callbacks)
```yuescript
-- Reverse arrow as last parameter
x <- f
body...

-- With placeholder
x <- map _, [1, 2, 3]
x * 2

-- Fat arrow backcall
<= f
body...
```

## 8. Function Stubs
Bind a method to its object:
```yuescript
run_callback my_object\write
-- equivalent to:
-- func = (self) -> self\write
```

## 9. Multi-line Arguments
```yuescript
my_func arg1, arg2, arg3,
  arg4, arg5, arg6
  -- continued at same indent
```

## 10. Implicit Objects
Arguments without explicit key — for function calls:
```yuescript
func
  * 1
  * 2
  * 3
-- passes *1*, *2*, *3* as separate arguments
```
