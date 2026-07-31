# YueScript Primer — Assignment

## 1. Basic Assignment

```yuescript
x = 10           -- local variable (default)
a, b, c = 1, 2, 3  -- multi-assignment
x = 20           -- reassign existing local
```

### Perform Update
```yuescript
x += 1  -- x = x + 1
s ..= "world"  -- string concat
arg or= "default"  -- only if arg is nil
```

### Chaining Assignment
```yuescript
a = b = c = 0  -- all get same value
```

### Explicit Scope
```yuescript
local x = 1
local `*`       -- forward declare all as local
local `^`       -- forward declare uppercase as local
global y = 2
global `*`      -- all as globals
```

## 2. Destructuring Assignment

### Array Destructuring
```yuescript
thing = [1, 2, 3]
[a, b] = thing       -- a=1, b=2
[a, _, c] = thing    -- skip second
[first, ...rest] = thing  -- rest = [2, 3]
```

### Table Destructuring
```yuescript
obj = {hello: "world", day: "Tuesday"}
{hello: hello, day: the_day} = obj  -- rename fields
{hello: hello} = obj               -- : shorthand
:day = obj  -- simple field extraction (no braces needed)
```

### Nested Destructuring
```yuescript
{numbers: [first, second], properties: {color: color}} = obj
```

### Default Values
```yuescript
{:name = "anon", :job = "unemployed"} = person
```

### Import-Style Destructuring
```yuescript
{:a, :b} = tbl       -- shorthand for {a: a, b: b}
{random: rand} = math  -- rename
```

### Range Destructuring
```yuescript
[first, ...bulk, last] = orders
[first, ...rest] = orders     -- everything after first
[...start, last] = orders     -- everything before last
[first, ..._, last] = orders  -- capture first & last, skip middle
```

### Destructuring in For Loops
```yuescript
for [left, right] in *tuples
  print left, right
```

## 3. If Assignment (Walrus Operator)

Assign inside condition; variable scoped to the `if` / `elseif` body:

```yuescript
if user := db.find_user "moon"
  print user.name
elseif hello := os.getenv "hello"
  print hello
else
  print "nobody"
```

### While with Assignment
```yuescript
while byte := stream\read_one!
  process byte
```

## 4. Varargs Assignment

```yuescript
ok, ... = fn(...)
count = select '#, ...
first = select 1, ...
```

## 5. Using Clause — Prevent Destructive Assignment

```yuescript
-- Prevent modifying closed variable `i`
my_func = (x using i) ->
  i = 10  -- creates new local, does NOT modify outer `i`
```

```yuescript
-- Allow modifying `i`, but nothing else
my_func = (x using nil) ->
  i = "new"  -- creates new local
```
