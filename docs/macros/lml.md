# `lml!`

> Generating Macro

The `lml!` macro (Lulu Markup Language) brings JSX-like syntax to Lua, allowing you to write declarative, XML-like structures directly in your code. This is an experimental feature and can be incredibly powerful for building UI components, configuration trees, or any other nested data structure.

## Basic Syntax

You write `lml!` followed by a block containing your markup.

```lua
local my_component = lml! {
  <box prop="some_value" id={123}>
    <button text="Click Me" />
    <CustomElement />
  </box>
}
```

## How it Works

The `lml!` macro transpiles this syntax into a series of nested function calls to a function you must define yourself named `lml_create`.

The example above would be converted into the following Lua code at compile time:

```lua
lml_create("box", { prop = "some_value", id = 123 },
  lml_create("button", { text = "Click Me" }),
  lml_create(CustomElement, {})
)
```

### The `lml_create` Function

You are responsible for implementing the `lml_create` function. Its signature is typically `lml_create(element, props, ...children)`.

- `element`: The name of the element (e.g., `"box"`) or a reference to a component function (e.g., `CustomElement`).
- `props`: A table of the attributes passed to the element.
- `...children`: A variable number of arguments representing the child elements.

This design makes `lml!` framework-agnostic. You can use it to generate data for any UI library or data structure you design.

### Example Implementation

Here is a very basic implementation that simply prints the structure.

```lua
function lml_create(element, props, ...)
  local children = {...}
  print(f"Element: {element}")
  print("Props:")
  fprint(props)
  print("Children Count: " .. #children)
  return { tag = element, props = props, children = children }
end

local my_ui = lml! {
  <window title="My App">
    <button text="OK" />
  </window>
}
```

## Dynamic Content

You can embed regular Lua code, including loops and function calls, inside your `lml!` block by wrapping it in `{}`.

```lua
local items = { "one", "two", "three" }

local list = lml! {
  <list>
    {
      -- This is regular Lua code
      Vec(items):map(function(item)
        return lml!{ <item text={item} /> }
      end).items
    }
  </list>
}
```
