# Lulu Macro System

Lulu now includes a powerful macro system that allows you to define reusable code templates. Macros are expanded at compile-time, providing zero-runtime overhead metaprogramming capabilities.

## Syntax

### Macro Definition

```lua
macro {
  macro_name ($param1, $param2, $param3) {
    -- macro body with parameter substitution
    -- use $param1, $param2, etc. to reference parameters
  }
}
```

### Macro Invocation

```lua
macro_name! arg1, arg2 {
  -- block argument (optional)
}
```

## Examples

### Simple Macro

```lua
macro {
  hello ($name) {
    print("Hello, " .. $name .. "!")
  }
}

hello! "World"
-- Expands to: print("Hello, " .. "World" .. "!")
```

### For-Each Loop Macro

```lua
local items = {0, 5, 10}

macro {
  for_each ($key, $iterator, $expr) {
    for $key in ipairs($iterator) do
      $expr
    end
  }
}

for_each! item, items {
  print(item)
}

-- Expands to:
-- for item in ipairs(items) do
--   print(item)
-- end
```

### Conditional Macro

```lua
macro {
  when ($condition, $then_block) {
    if $condition then
      $then_block
    end
  }
}

when! x > 5 {
  print("x is greater than 5")
}

-- Expands to:
-- if x > 5 then
--   print("x is greater than 5")
-- end
```

### Function Definition Macro

```lua
macro {
  def ($name, $params, $body) {
    function $name($params)
      $body
    end
  }
}

def! add, a, b {
  return a + b
}

-- Expands to:
-- function add(a, b)
--   return a + b
-- end
```

### Class-like Macro

```lua
macro {
  class ($name, $methods) {
    $name = {}
    $name.__index = $name
    
    function $name:new()
      local instance = setmetatable({}, $name)
      return instance
    end
    
    $methods
  }
}

class! Person {
  function Person:speak()
    print("Hello!")
  end
}

-- Expands to a complete class definition with constructor
```

## Features

### Parameter Substitution
- Parameters are prefixed with `$` in macro definitions
- Parameters can be identifiers, strings, numbers, or code blocks
- Block arguments (the `{}` part) are passed as the last parameter

### Multiple Parameters
- Macros can accept multiple parameters separated by commas
- Parameters are matched by position during expansion

### Block Arguments
- The `{}` block after macro invocation is treated as a special argument
- Useful for creating control flow macros like loops and conditionals

### Compile-time Expansion
- Macros are expanded during compilation, before runtime
- No performance overhead compared to writing the expanded code directly

## Advanced Examples

### Repeat Macro

```lua
macro {
  repeat_n ($times, $body) {
    for i = 1, $times do
      $body
    end
  }
}

repeat_n! 3 {
  print("Hello!")
}
```

### Try-Catch Style Error Handling

```lua
macro {
  try_catch ($try_block, $catch_block) {
    local ok, err = pcall(function()
      $try_block
    end)
    if not ok then
      $catch_block
    end
  }
}

try_catch! {
  error("Something went wrong")
} {
  print("Caught error:", err)
}
```

### Getter/Setter Generation

```lua
macro {
  property ($obj, $name, $field) {
    function $obj:get_$name()
      return self.$field
    end
    
    function $obj:set_$name(value)
      self.$field = value
    end
  }
}

property! MyClass, name, _name
-- Generates get_name() and set_name() methods
```

## Implementation Details

The macro system is implemented in `src/compiler.rs` with the following components:

1. **Lexer Extensions**: Added tokens for `macro`, `MacroCall`, `MacroParam`, and structural tokens
2. **Macro Registry**: Stores defined macros with their parameters and bodies
3. **Parser**: Parses macro definitions and stores them in the registry
4. **Expander**: Expands macro calls by substituting parameters
5. **Code Generator**: Outputs the final expanded code

## Usage Tips

- Use macros for repetitive code patterns that can't be easily abstracted with functions
- Macro parameters can be any valid Lua tokens (identifiers, strings, numbers, operators)
- The block argument is powerful for creating domain-specific languages
- Macros are expanded in the order they appear in the source code
- A macro must be defined before it can be used

## Compilation

Macros are processed during the compilation phase:

```bash
# The compiler automatically handles macro expansion
lulu run my_script_with_macros.lua
```

The expanded code is what actually gets executed, so debugging will show the expanded version rather than the original macro calls.