# YueScript Primer — Module System

## 1. Import

### Import with destructuring
```yuescript
import a, b from "module"
import :x, :y from "module"  -- shorthand for {x: x, y: y}
import "module" as alias      -- require entire module as alias
import "module" as {deep: {key}} -- table destructuring
import "module" as {name: renamed} -- rename
```

### Import all from package
```yuescript
import "package" as :Class, :function
```

### Import global (implicit import of globals)
```yuescript
do
  import global
  print "auto-imports print as local const"
  math.random 3  -- auto-imported as local const
```

### Python-style import
```yuescript
from "module" import a, b, c
```

### Simple require
```yuescript
import "module"
import "module.name"
import 'module-name'
import "module-x"  -- hyphenated paths
```

## 2. Export

### Named Export
```yuescript
export a = 1
export b = 2
export c = if condition then "yes" else "no"
export class MyClass
  body
export default some_value  -- replace exported table
```

### Destructured Export
```yuescript
export {fieldA: :fieldB} = table
export :name = local_value
```

### Index Export (no local name)
```yuescript
export.key = value
export["index-key"] = value
export.<field> = value
```

### Unnamed Export
```yuescript
export value1
export value2
export if condition then 1 else 2
export with obj
  .field = 1
```

## 3. Module Loading Flow
1. `import "modname"` → looks for `modname.yue`
2. Searches module paths
3. Loads and compiles `modname.yue`
4. Exports return the `export` table of the loaded module

## 4. Usage Pattern
```yuescript
-- module.yue
export add = (a, b) -> a + b
export class Counter
  new: =>
    @n = 0

-- consumer.yue
import :add, :Counter from "module"
counter = Counter!
result = add(1, 2)
