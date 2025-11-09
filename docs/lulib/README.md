# Lulibs

A lulib is a format to store your lua bundle, basically a bundle of bytecode made up of config data (such as config manifest) and the bytecode of every lua module included in the stack at module resolution in lulu.

## The Lulib Stack

Lulibs have a stack that would look somewhat like this:
```
configs:
  - [Config Data]
    -> manifest.name = myproject
modules:
  - myproject/init
    => [Bytecode Data]
  - myproject/utils
    => [Bytecode Data]
  - myproject/src-core
    => [Bytecode Data]
```
If you have included multiple lulibs, this is what it would look like:
```
configs:
  - [Config Data 1]
    -> manifest.name = myproject
  - [Config Data 2]
    -> manifest.name = somelib
modules:
  - myproject/main
    => [Bytecode Data]
  - somelib/init
    => [Bytecode Data]
  - somelib/src-core
    => [Bytecode Data]
```
This is why you can load multiple lulibs into one lulib.

## Including Lulibs

To include lulibs, you will have to include each lulib as mentioned in [the include field](../reference/configuration.md#include). Before you include the lulib, make sure the lulib has a a package name (`manifest.name`) and the entry module called `init`.

## Loading Lulibs

The simplest way to load a lulib is to index exactly what the lulib name is and the module name.

```lua
local mymod = require("package_name/init")
```

However, if you have the `package_name/init`, loaded in stack, you can use it as such:

```lua
using {
  -- this way, you can request_load_env and load this into the
  -- current module from the stack
  lulib.package_name
}
```

Following this rule, you can also dynamically include lulibs as:
```lua
using {
  -- this will cahce the url, and register
  -- the lulib name into the current global
  lulib.from "github:username/repo"
}

repo.do_something()

--- or you can use require_cached/require_cached_async
local repo = require_cached("github:username/repo")
```

## Loading modules into a lulib

You can load modules into a lulib from files in 3 ways.
- **Module Indexing**: By mapping each module inside of your `lulu.conf.lua` as mentioned in [the config](/reference/configuration.md#mods), you can resolve each module at entry and add them to the stack.
- **Macro importmap**: You can use the [`import!` macro](/macros/other-macros.md#import) to load a module dynamically at compile time, this way each imported module would be collected by the macro recursively.
<li>
  <div class="side-by-side-list">

  <div style="side-by-side-content">

  **`Using` directive**: By using `using`, you can link files and add them to the stack but only after macro importmaps have been resolved.

  </div>

  <div style="side-by-side-list-code">

  ```lua
    using.lulib { utils, "./utils.lua" }
    -- or
    using {
      lulib { utils, "./utils.lua" }
    }
  ```

  </div>

  </div>
</li>