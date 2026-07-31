# YueScript Primer — Data Structures

## 1. Array Literals (List Literals)
```yuescript
list = [1, 2, 3]
empty = []
single = [1,]
```

## 2. Table Literals (Hash Tables)
```yuescript
table = {
  key1: "value1"
  key2: 123
}

-- Braceless (when value is a single key-value table)
obj = height: 10, width: 20

-- With computed keys
tbl = {
  key1 "default_value"
  [expr]: value
}

-- With : prefix (shorthand)
:hair, :height   -- {hair: hair, height: height}

-- Spreading
merge = {...a, ...b}
```

### Spread in Arrays vs Tables
```yuescript
a = {...other}   -- copies array + hash parts
list = {...a,}   -- ONLY array part
```

### Implicit Objects in Tables
```yuescript
tb =
  name: "Bob"
  values:
    - "a"
    - "b"
  objects:
    - x: 1
    - x: 2
```

## 3. Table Comprehensions
```yuescript
-- Table comprehension
copy = {k, v for k, v in pairs tb}
filtered = {k, v for k, v in pairs tb when v > 0}

-- With *: numeric iteration
sqrts = {i, math.sqrt i for i in *numbers}

-- Single expression producing key+value
tbl = {unpack item for item in *tuples}
```

## 4. List Comprehensions
```yuescript
-- Basic
doubled = [item * 2 for item in *items]

-- With filter
evens = [x for x in *numbers when x % 2 == 0]

-- Nested (multiple for clauses)
pairs = [[x, y] for x in *xs for y in *ys]

-- Numeric
evens = [i for i = 1, 100 when i % 2 == 0]

-- Flatten nested with spread
flat = [...v for k, v in pairs nested]
```

## 5. Slicing
```yuescript
-- Slice expression (returns a new table)
sub = items[2, 4]         -- items 2 through 4
sub = items[-4, -1]       -- last 4 items
sub = items[1,, 2]        -- every 2nd item starting at 1
sub = items[,-1, -1]      -- reversed
```
