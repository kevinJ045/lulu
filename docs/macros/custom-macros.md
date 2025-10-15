# Custom Macros

Lulu allows you to define your own macros, which can then be used within your project or exported for other projects to use.

## Defining a Macro

You define macros using the `macro` keyword. A definition consists of the macro name, a list of parameters, and a body.

- **Parameters**: Prefixed with a `$` (e.g., `$param`). An underscore prefix (`$_param`) marks a parameter as optional.
- **Body**: The code that will be generated. You can use the parameters within the body.

```lua
-- This file can be named anything, e.g., _macros.lua

macro {
  -- Defines a macro named `add` with two required parameters
  add ($num1, $num2) {
    $num1 + $num2
  }
}

macro {
  -- Defines a macro named `hello` with one optional parameter
  hello ($_name) {
    print("Hello, " .. ($_name or "World"))
  }
}
```

## Using Custom Macros

Once defined, you can use them like any other macro.

```lua
local sum = add! 2, { 3 } -- The last argument must be in braces
print(sum) -- Output: 5

hello! {} -- Output: Hello, World
hello! { "Makano" } -- Output: Hello, Makano
```

## Exporting Macros

To make your macros available to other projects that depend on yours, you must export them in your `lulu.conf.lua` file using the `macros` field.

This field can be a string containing the macro definitions, or you can load them from a file.

```lua
-- lulu.conf.lua

-- Option 1: Inline string
macros = [[
  macro {
    add ($num1, $num2) { $num1 + $num2 }
  }
]]

-- Option 2: Load from a file (recommended)
macros = io.open("_macros.lua"):read("*a")
```

Now, if another project adds `"github:your/project"` to its dependencies, it will be able to use the `add!` macro.
