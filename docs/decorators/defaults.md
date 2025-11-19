# STD Decorators

Lulu comes with a few built-in decorators to help you simplify tasks.

## `default_to`

> Param Decorator

Helps set a default.

```lua
local {
  @default_to(0)
  x,
  @default_to(0)
  y
} -> Vec2

local u = Vec2()
u.x -- 0

local v = Vec2(10, 10)
v.x -- 10
```

## `map_into`

> Param Decorator

Maps a value into another one.

```lua
local {
  @map_into(function(val)
    return f"prefix-{val}"
  end)
  prefixed,


  @map_into(function(table)
    return Vec(table)
  end)
  vec
} -> Struct
```

## `default_not_nil`

> Param Decorator

Throws an error when the value is not provided

```lua
local {
  @default_not_nil
  name,
} -> Person
```

## `validate_type`

> Param, Enum Variant and Function Decorator

Validates the type of values.

```lua
local {
  @validate_type('string')
  name,
  @validate_type('number')
  id
} -> Person

@validate_type('number', 'string')
function myFunc(num, str)

end

local {
  @validate_type('number', 'string')
  Variant(num, str)
} -< MyEnum
```

## `globalize`

> Function Decorator

Globalizes given function.

```lua
@globalize
function theFunction()

end

-- in another module
theFunction()
```