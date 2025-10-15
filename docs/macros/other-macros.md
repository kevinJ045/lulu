# Other Macros

Here are a few default macros to help you out.

## `import!`

> Transforming Macro

Lets you include a Lua file as a module at compile time, similar to adding it to the `mods` table in your `lulu.conf.lua`.

```lua
-- This line...
import! utils, { "./utils.lua" }

-- ...is compiled into this:
local utils = require("utils")
```

Lulu automatically assigns a module name based on the file path (e.g., `src/api/client.lua` becomes `src-api-client`).

This macro is specially useful in case you don't want to add these files into your `lulu.conf.lua` manually, meanwhile also preserving the config size for fetches.

## `include_bytes!`

> Generating Macro

Includes the entire content of a file as a raw byte sequence (as a Lua table). This is useful for embedding assets like images, sounds, or text files directly into your executable.

```lua
include_bytes! my_asset, { "./assets/icon.png" }

-- `my_asset` is now a string containing the binary data of the file.
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

## `repeat_n!`

> Generating Macro

A simple for loop that repeats a block of code `n` times.

```lua
-- Prints numbers from 1 to 5
repeat_n! 1, 5, {
  print(i) -- The iterator `i` is automatically available
}
```
