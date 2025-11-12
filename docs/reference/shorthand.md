# Shorthand Expressions

Shorthands are basically simple expressions invalid in native lua that will be compiled into a complex valid lua structure through lulu.

## Function Shorthands

Simple arrow functions that allow decoration. 


<table class="side-by-side">
<tr>
<th>Shorthand</th>
<th>Lua</th>
</tr>
<tr>
<td>

```lua

-- Basic:
local myFunc = () =>
  print("Hi")
end

-- Named
() myFunc =>
  print("Hi")
end

local () myFunc =>
  print("Hi")
end

-- Methods
() myClass:myFunc =>
  print("Hi")
end

-- Async
() async =>
  print("Hi")
end

-- Decorated
local ()
@depricated
@validate("string") myFunc =>
  print("Hi")
end

-- Params Decorated
local (a, @default_to(0) b) myFunc =>
  print("Hi")
end

-- Namespaced
() @namespace(object) =>
  print("Hi")
end

```
</td>
<td>

```lua

-- Basic:
local myFunc = function()
  print("Hi")
end

-- Named
function myFunc()
  print("Hi")
end

local function myFunc()
  print("Hi")
end

-- Methods
function myClass:myFunc()
  print("Hi")
end

-- Async
async(function()
  print("Hi")
end)

-- Decorated
-- Not possible





-- Params Decorated
-- Not possible



-- Namespaced
namespace(object)(function()
  print("Hi")
end)
```
</td>
</tr>
</table>

## Class Shorthands/Structs

Allows you to create simple structs that also allow for inheritance. However it's meant mainly for bodyless classes. Uses the [`class!` macro](../macros/class.md).


<table class="side-by-side">
<tr>
<th>Shorthand</th>
<th>Class</th>
</tr>
<tr>
<td>

```lua
-- Basic
{
  name,
  id
} -> Person

-- Named
{
  #name,
  #id
} -> Person

-- Local
local {
  name,
  id
} -> Person

-- Inheritance
local {
  name,
  id
} -> Person:Parent

-- Decorated
local {
  @validate_type("string")
  name,
  @default_to(0)
  id
} -> Person

-- Methods
local {
  name,
  id
} -> Person

() Person:init =>
  ...
end

-- Constructor
-- Not available












```
</td>
<td>

```lua
-- Basic
class! Person(
  name,
  id
);

-- Named
class! Person(
  #name,
  #id
);

-- Local
local class! Person(
  name,
  id
);

-- Inheritance
local class! Person:Parent(
  name,
  id
);

-- Decorated
local class! Person(
  @validate_type("string")
  name,
  @default_to(0)
  id
);

-- Methods
local class! Person(
  name,
  id
), {
  init(){
    ...
  }
}

-- Constructor
local class! Person(
  name,
  id
), (name, id){
  -- Mostly useful to call super
  -- You can use init otherwise
  super(name)
}, {
  init(){
    ...
  }
}
```
</td>
</tr>
</table>

## Enum Shorthands

A simpler way to make enums. Uses the [`enum!` macro](../macros/enum.md).


<table class="side-by-side">
<tr>
<th>Shorthand</th>
<th>Enum</th>
</tr>
<tr>
<td>

```lua
-- Basic
local {
  String(content),
  EOF
} -< Token

-- Decorators
local {
  @validate_type("string")
  String(content),
  EOF
} -< @decorator Token

-- Enum Methods
local {
  String(content),
  EOF
} -< Token

Token.func.get_string = (item) =>
  return item.content
end
```
</td>
<td>

```lua
-- Basic
local enum! Token, {
  String(content),
  EOF
}

-- Decorators
local enum! @decorator Token, {
  @validate_type("string")
  String(content),
  EOF
}

-- Enum Methods
local enum! Token, {
  String(content),
  EOF
}, {
  get_string(item){
    return item.content
  }
}

```
</td>
</tr>
</table>

## Namespace shorthand

You can use any table as a namespace in lulu like this:
```lua

local my_table = {
  x = 10
}

namespace(my_table)(function(this)
  print(x)
end)

-- or

() @namespace(my_table) =>
  print(x)
end
```

However, you can simply do this for a dynamic, contained, multi-namespaced environment.

```lua
in local new_namespace do
  name = 1
end

-- access it as:
print(new_namespace::name)

-- to use multiple namespaces:
local ns = {
  x = 1
}

local ns2 = {
  y = 2
}

in local my_namespace and ns and ns2 do
  print(x, y) -- 1, 2

  y = y + 10
end

print(ns2.y) -- 2
print(my_namespace::y) -- 12

-- quick namespaces
in local _ and ns do
  -- this is if you
  -- are only using
  -- namespaces and
  -- do not want to
  -- create a new one
  print(f"x is {x}")
end
```
## Expression shorthands

In lua, you can't have blocks where a value would go. In Lulu, you can as such:

```lua

local myval = in do
  -- you can do whatever you want here,
  local my_sub_val = "something"
  return my_sub_val
end

-- quick ifs

local myval = in if true then
  return "foo"
else
  return "bar"
end

-- or
print(in if true then ... else ... end)


-- quick for/while

local my_table = in for i = 1, 10 do
  collect(i) -- collect only exists in this block. 
             -- "i" will be a part of the result table
end

local i = 0
local my_other_table = in while i < 10 do
  i += 1
  collect(i)
end
```

## Operation shorthands

In lua, you can't use `+=`, `-=`, `*-`, `/=` and `!=`. But lulu has the sugar to allow for these.

```lua
local x = 10

x += 2
x -= 2
x *= 2
x /= 2

if x != 20 then
  print(x) -- 10
end

-- works in objects too:

local o = { x = 10 }

o.x += 10

print(o.x) -- 20
```

## String Formatter

Format strings dynamically using embedded Lua expressions with `f"..."` syntax:

```lua
-- Example
local name = "John"
local score = 42

local result = f"Player {name} scored {score * 2} points! {{escaped braces}}"

-- Translates to:
local result = "Player " .. (name) .. " scored " .. (score * 2) .. " points! {escaped braces}"
```

## Pointer Shorthands

Simple access to simulated pointers.

```lua
-- Creating
local strPtr   = &"Hello, World!"
local numPtr   = &42
local boolPtr  = &true
local varPtr   = &someVar

print(strPtr) 
-- → number (memory address of Rust container)

-- Dereferencing
local strVal = *strPtr
print(strVal) 
-- → "Hello, World!"

*numPtr = 100       -- change the value via the pointer
print(*numPtr)      -- → 100

-- Manual management
local ptr = ptr_of("initial value")   -- create a pointer manually

ptr_set(ptr, "updated value")         -- set a new value
local val = ptr_deref(ptr)            -- get the value

print(val)                            -- → "updated value"
```