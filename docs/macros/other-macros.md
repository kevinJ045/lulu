# Other Macros

Here are a few default macros to help you out.

## `import!`

> Transforming Macro

Lets you include a Lua file as a module at compile time, similar to adding it to the `mods` table in your `lulu.conf.lua`.

```lua
-- This line...
import! utils, { "./utils.lua" }
-- or ';' terminated
import! utils, "./utils.lua";

-- ...is compiled into this:
local utils = require("utils")
```

Lulu automatically assigns a module name based on the file path (e.g., `src/api/client.lua` becomes `src-api-client`).

This macro is specially useful in case you don't want to add these files into your `lulu.conf.lua` manually, meanwhile also preserving the config size for fetches.

## `include_bytes!`

> Generating Macro

Includes the entire content of a file as a raw byte sequence (as a [ByteArray](../reference//helper-classes.md#bytearray)). This is useful for embedding assets like images, sounds, or text files directly into your executable.

```lua
local my_asset = include_bytes! { "./assets/icon.png" }
-- or ';' terminated
local my_asset = include_bytes! "./assets/icon.png";
-- you can also include strings as
local txtfile = include_string! "./assets/file.txt";

-- `my_asset` is now a ByteArray containing the binary data of the file.
-- `txtfile` is now a string containing the file text.
```

## `for_each!` & `for_pairs!`

> Generating Macros

These are simple convenience macros for writing `for` loops.

```lua
local items = { "a", "b", "c" }

-- Equivalent to: for item in ipairs(items) do ... end
for_each! item, items, {
  print(item)
}

local my_table = { key = "value" }

-- Equivalent to: for k, v in pairs(my_table) do ... end
for_pairs! k, v, my_table, {
  print(k, v)
}
```

## `when!`

> Generating Macro

A simple if/else expression.

```lua
local x = 10

-- Simple if
when! x > 5, {
  print("x is greater than 5")
}

-- If/else
when! x > 20, {
  print("x is greater than 20")
}, {
  print("x is not greater than 20")
}
```

## `try_catch!`

> Generating Macro

Wraps a block of code in `pcall` to safely handle errors.

```lua
try_catch! {
  -- Code that might fail
  error("something went wrong")
}, {
  -- This block executes on failure
  -- The error message is available in the `err` variable
  print(f"Caught an error: {err}")
}
```

## `guard!`

> Generating Macro

Checks if a condition is true and throws an error if it's not.

```lua
guard! {
  user.is_admin == true
}, {
  "User does not have admin rights!"
}
```

## `collect!`

> Generating Macro

Collects variables into a table.

```lua
collect! {
  name,
  id,
  something = some_other_thing
}
-- into
{
  name = name,
  id = id,
  something = some_other_thing
}

-- for objects:

collect! {
  name,
  id,
  ..array,
  ...some_table
}

-- into

(function()
  local _t = {
    name = name,
    id = id
  }
  for k,v in pairs(array) do
    _t[k] = v
  end
  for k,v in ipairs(some_table) do
    _t[k] = v
  end
  return _t
end)()
```

## `spread!`

> Generating Macro

Spreads an array/table into local variables.

```lua
-- while you can do:
local first, _, third = unpack({ 1, 2, 3 })

-- you can do this for more complex spreading
spread! mytable, {
  first,
  ...last
}

-- into
local first = mytable[1]
local last = { unpack(mytable, 2, #mytable) }

-- you can do all these:
spread! mytable, {
  first,
  _, -- skipped,
  ...thing,
  last_1,
  &named,
  named_by_key: name,
  last_2
}

-- into
local first = mytable[1]
local thing = { unpack(mytable, 3, #mytable - 3) }
local last_1 = mytable[4]
local named = mytable.named
local named_by_key = mytable.name
local last_2 = mytable[5]
```

## `repeat_n!`

> Generating Macro

A simple for loop that repeats a block of code `n` times.

```lua
-- Prints numbers from 1 to 5
repeat_n! 1, 5, {
  print(i) -- The iterator `i` is automatically available
}
```
