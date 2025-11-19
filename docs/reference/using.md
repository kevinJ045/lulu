# The Using Directive

The `using` directive is a function that allows you to control the current module's environment/context.


# Using `lulib/dylib`

The `lulib` and `dylib` namespaces/items/functions provide `Usage`s that import either a dynamic library or a lulu library to every context. More of that in [Lulibs](/lulib/README.md)

# Using Usages

A usage is a function that takes the current context and modifies as such:

```lua
@Usage
function MyUsage(ctx, data)
  -- data is shared across all usages
  data.global -- all global usage data
  data.mod -- usage data for this mod

  -- ctx is the module context of which 
  -- this usage is used.
  ctx.myvar = "something"

  print(ctx.mod.name)
end

using {
  MyUsage
}

print(myvar) -- something
```

# Using `static` and `keystore`

`static` allows you to set a static variable once per runtime and modify it as well.

```lua
using {
  static "my_static" (0)
}

-- Everytime this module is ran(or imported)
-- my_static will increase
my_static += 1
print(my_static)
```

`keystore` allows you to store values beyond the module's context.

```lua
using {
  keystore
}

kset('myval', 0)

kget('myval') -- 0
```

# Using `namespace`

You can append namespaces into your current context with `using`.

```lua
-- If we had these two objects
local x = {
  var = "the value"
}

local y = {
  value = 0
}

-- Normally, you'd use `namespace` like this:

@namespace(x, y)
function()
  print(var)
end
-- or

() @namespace(x, y) =>
  print(var)
end

-- However, by doing this, you can use them in top-level
using {
  namespace(x, y)
}

print(var) -- "the value"
print(value) -- "0"
```