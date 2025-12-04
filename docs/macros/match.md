# `match!`

> Generating Macro

The `match!` macro provides exhaustive pattern matching, similar to a `switch` statement in other languages but with a few more functionalities. It is especially useful when paired with enums created by [`enum!`](../macros/enum.md).

## Basic Matching

`match!` takes a value to match against, followed by a block of branches. Each branch consists of a pattern and a block of code to execute if the pattern matches.

```lua
local value = "case_b"

match! value, {
  "case_a" {
    print("Matched A")
  }
  "case_b" {
    print("Matched B")
  }
  -- Multiple cases
  "case_c" or "case_d" {
    print("Either C or D")
  }
  -- Negatives
  not "case_e" {
    print("Not E")
  }
  -- The `_` pattern is a wildcard that matches anything
  _ {
    print("Default case")
  }
}
-- Output: Matched B
```

## Matching Enum Variants

The primary use case for `match!` is to handle all possible variants of an enum.

```lua
enum! MyResult, { Success(result), Failure(error) }

local result = MyResult.Success("Data loaded")

match! result, {
  MyResult.Success {
    -- The `val` variable holds the enum instance
    -- and its contents can be accessed by name
    print(f"Success: {val.result}")
  }
  MyResult.Failure {
    print(f"Error: {val.error}")
  }
}
-- Output: Success: Data loaded
```

## Using `match!` as an Expression

If you use the `return` keyword inside the branches, the `match!` macro can be used as an expression to return a value.

```lua
local message = match! result, {
  MyResult.Success {
    return f"Success: {val.result}"
  }
  MyResult.Failure {
    return f"Error: {val.error}"
  }
}

print(message)
```

## Custom Conditions

You can also provide a custom boolean expression as a match condition.

```lua
local x = 10

match! x, {
  (val > 5) {
    print("x is greater than 5")
  }
  (val <= 5) {
    print("x is 5 or less")
  }
}
```


## Match shorthand

A simple match statement syntax normalizer.

```lua
local x = 10

match(x) do
  if (val > 5) then
    return "it is five!"
  if 3 or 4 then
    return "almost there!"
  if _ then
    return "very small"
end
```
