# `enum!`

> Generating Macro

The `enum!` macro brings algebraic data types to Lua, similar to enums in Rust. This allows you to define a type that can be one of several different variants, each potentially holding data.

## Defining an Enum

Define an enum with a name and a list of variants. Variants can either be simple (no data) or tuple-like (containing data).

```lua
enum! WebEvent, {
  PageLoad, -- Simple variant
  KeyPress(string), -- Tuple-like variant with one value
  Click(x, y) -- Variant with a table
}
```

## Creating Instances

You create an instance by calling the variant like a function.

```lua
local event1 = WebEvent.PageLoad
local event2 = WebEvent.KeyPress("k")
local event3 = WebEvent.Click({ x = 10, y = 20 })
```

## Using Enums with `match!`

Enums are most powerful when combined with the [`match!`](../macros/match.md) macro to handle each possible variant.

```lua
local function handle_event(event)
  match! event, {
    WebEvent.PageLoad {
      print("Page loaded.")
    },
    WebEvent.KeyPress {
      -- The `val` variable holds the enum instance
      print(f"Key pressed: {val.string}")
    },
    WebEvent.Click {
      print(f"Clicked at {val.x}, {val.y}")
    }
  }
end

handle_event(event2) -- Output: Key pressed: k
```

## Enum Methods

You can also define methods on an enum, which can be called on any of its instances.

```lua
enum! MyEnum, {
  Variant(content)
}, {
  -- Define methods in a second block
  unwrap(item) {
    return item.content
  }
}

local instance = MyEnum.Variant("hello")
print(instance:unwrap()) -- Output: hello
```
