# YueScript Primer — Objects

## 1. Class Declaration
```yuescript
class ClassName
  new: (constructor_args) =>
    @field = value

  method_name: (args) =>
    @field  -- instance field
```

### Create Instance
```yuescript
obj = ClassName!
obj\method args  -- call method with self
```

## 2. `@` and `@@`
- `@` = instance (self)
- `@@` = class (self.__class)

### Field Assignment
```yuescript
new: (@foo, @bar) =>       -- constructor parameter promotion
```

### Accessing class from instance
```yuescript
@@count  -- instance field access
@@\class_method  -- call class method
```

## 3. Inheritance
```yuescript
class Child extends Parent
  method_name: =>
    super "arg"  -- call parent method
    super.parent_field  -- access parent field

  @static_field: value  -- class-level field
```

### `super` Forms
```yuescript
super args        -- call parent method (self auto-inserted)
super\method args -- same (stub form)
super.field       -- access parent field directly
== ParentClass   -- super is the parent class
```

## 4. `__inherited` Hook
```yuescript
class Parent
  @__inherited: (child) =>
    print "#{@__name} -> #{child.__name}"

class Child extends Parent
-- prints: "Parent -> Child"
```

## 5. Class Objects
Every class creates a class object:
- `ClassName` — the class object itself
- `obj.__class` — reference to class
- `ClassName.__name` — class name string
- `ClassName.__base` — base prototype table
- `ClassName.__parent` — parent class (if inherited)

### Class-Level Variables
```yuescript
class Counter
  @count = 0  -- class variable (on class object, not instances)
  @increment: => @@count += 1
```

## 6. `with` Block
Access an object without repeating its name:
```yuescript
with obj
  .field = value     -- set field
  \method args       -- call method
  [key] = value      -- index access
  print .field       -- read field

-- with? for nil-safe
with? maybe_nil
  \do_something!
```

### `with` as expression
```yuescript
obj = with NewObject!
  .init!
  \configure!
```

## 7. Class Expressions (Anonymous)
```yuescript
cls = class extends Base
  method: => @

-- Anonymous with name from LHS
BigClass = class extends Small
```

## 8. Class Mixing
```yuescript
class MyClass using MixinTable
  body...

class MyClass using ExistingClass  -- shallow copy of methods only
  body...
