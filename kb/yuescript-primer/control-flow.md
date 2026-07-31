# YueScript Primer — Control Flow

## 1. Conditionals

### if / else / elseif
```yuescript
if condition
  expr1
elseif condition2
  expr2
else
  expr3
```

### Inline (one-liner)
```yuescript
x = condition then "yes" else "no"
"hello" if condition
"hello" unless condition  -- inverse of if
```

### if as expression (returns a value)
```yuescript
message = if is_valid
  "valid"
else
  "invalid"
```

### in expression
```yuescript
if x in [1, 2, 3]       -- membership check
if x in table            -- also works with tables
if x not in list         -- negated
```

### switch
```yuescript
switch value
  when "a"
    print "it is a"
  when "b", "c"       -- multiple matchers
    print "b or c"
  else
    print "else"
```

### switch as expression
```yuescript
result = switch x
  when 1 then "one"
  when 2 then "two"
  else "other"
```

### switch with table matching
```yuescript
switch item
  when :x, :y
    print "Vec2(#{x}, #{y})"
  when {width, height}
    print "size: #{width}x#{height}"
  when [1, b, 3]
    print "array match with b=#{b}"
  when success: true, :result
    print "success: #{result}"
```

### switch assignment
```yuescript
switch name := get_name!
  when "Bob"
    print "Bob"
```

## 2. For Loop

### Numeric for
```yuescript
for i = start, end
  body
for i = start, end, step
  body
```

### Generic for
```yuescript
for key, value in pairs tbl
  body
```

### Slicing in for
```yuescript
for item in *items[2, 4]
  body
```

### One-liner with `do`
```yuescript
for i = 1, 10 do print i
```

### For as expression (accumulator)
```yuescript
doubled = for i = 1, 10
  i * 2

first_found = for n in *numbers
  break n if n > 10   -- early exit with value
```

## 3. While / Until

```yuescript
while condition
  body

while condition do body   -- one-liner

until condition
  body
  -- exits when condition is true
```

### As expression
```yuescript
result = while true
  n = get_next!
  break n if n > 10
```

## 4. until
```yuescript
until condition
  body

-- exits when condition becomes truthy
```

## 5. repeat ... until
```yuescript
repeat
  body
until condition
-- always runs at least once
```

## 6. continue
```yuescript
for i = 1, 10
  continue if i % 2 == 0
  process i  -- only processes odd numbers
```

## 7. Goto & Labels

### Define labels
```yuescript
::my_label::
```

### Jump to label
```yuescript
goto my_label if condition
```

### Notes
- Lua 5.2+ only (goto not valid in Lua 5.1)
- Labels must be unique in scope
- Cannot jump into inner scopes
