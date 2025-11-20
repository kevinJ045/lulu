# Helper Classes

Lulu injects a few Rust-backed helper "classes" into the Lua runtime to make working with data structures more ergonomic and efficient. These classes provide rich APIs for data manipulation.

## `Vec`

A dynamic, array-like list.

### Creation
```lua
-- Create with the `vec!` macro for a chained-method style
local my_vec = vec! { 1, 2, 3 }
my_vec.push(4)

-- Or create with the constructor for a colon-method style
local my_vec2 = Vec({ 1, 2, 3 })
my_vec2:push(4)

-- You can convert between styles
my_vec2:into().push(5) -- Switch to `.` style
my_vec.collect():push(6) -- Switch to `:` style
```

### API

- **Mutation**: `push`, `insert`, `sort`, `reverse`, `pop`, `set`, `remove_at`, `remove`, `extend`
- **Access**: `len`, `get`, `find`, `items` (returns a raw Lua table)
- **Iteration**: `clone`, `map`, `for_each`, `filter`

## `String`

A wrapper around strings that provides a useful method chaining API.

```lua
local my_str = String("Hello, World!"):into()

my_str.push_str(" How are you?")
local parts = my_str.split(", ") -- returns a Vec
```

### API

- **Mutation**: `push_str` (appends a Lua string), `push_string` (appends another `String` instance)
- **Access**: `starts_with`, `ends_with`, `as_str` (returns a raw Lua string)
- **Iteration**: `clone`, `split`
- **Regex**: `match`, `replace`

## `Map` and `Set`

Lua-based implementations of a map (key-value store) and a set (unique values).

```lua
local my_map = Map():into()
my_map.set("key", "value")

local my_set = Set():into()
my_set.add("item1")
my_set.add("item1") -- This will only be stored once
```

### Map API
- `set(key, value)`, `remove(key)`, `get(key)`, `has(key)`, `keys()`, `values()`, `clone()`

### Set API
- `add(item)`, `remove(item)`, `has(item)`, `values()`, `clear()`, `clone()`

## `WeakMap` and `WeakSet`

These are versions of `Map` and `Set` that hold their keys weakly, allowing them to be garbage collected if there are no other references. Their APIs are identical to `Map` and `Set`.

## `HashMap` and `HashSet`

These are high-performance, Rust-backed implementations of a hash map and hash set. Unlike `Map` and `Set`, they can use tables and other complex types as keys reliably.

**Note**: These types cannot be cloned.

```lua
local map = HashMap()
local key = {}
map:set(key, "some value")
print(map:get(key)) -- prints "some value"

local set = HashSet()
set:add("value")
```

### HashMap API
- `:set(key, value)`, `:get(key)`, `:has(key)`, `:remove(key)`

### HashSet API
- `:add(value)`, `:has(value)`, `:remove(value)`, `:values()`, `:clear()`

## `ByteArray`

`ByteArray` is a simple Rust-backed byte management utility that manages a `Uint8Array` or `Vec<u8>`.

```lua
local mybytes = ByteArray({ ... })

print(mybytes:to_str()) -- into lua byte string
print(mybytes:to_string()) -- into utf-8 string
print(mybytes:to_table()) -- into lua table (too slow)
print(mybytes:len()) -- the length
```

### ByteArray API
- `:to_table()`, `:len()`, `:to_hex()`, `:to_string()`, `:clear()`, `:to_str()`
- `:copy()`, `:slice(start, stop)`, `:pop()`, `:push(byte)`, `:extend_table(lua_table)`, `:extend(bytearray)`, `:map(fn)`

## Memory Safety

- **`Arc(v)`**: Creates a simple `Arc` wrapped rust contained safe variable.
  -   **where v can be**:
      - `String`
      - `Number`
      - `Table`
      - `Dict/Table`
- **`ArcMutex(v)`**: An `Arc<Mutex<LuluWrappedValue>>` container
- **`ArcRwlock(v)`**: An `Arc<Rwlock<LuluWrappedValue>>` container
  -    **Usage**:
      ```lua
      local f = ArcMutex(0)
      -- getting
      f() or f:get()
      -- setting
      f(1) or f:set(1) -- to set

      f.type -- the arc type
      f.kind -- the kind
      f:tostring() -- rust formatted string
      f:clone_handle() -- clones the handle, same value
      ```