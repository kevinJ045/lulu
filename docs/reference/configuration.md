# Configuration (`lulu.conf.lua`)

The `lulu.conf.lua` file is the brains of your Lulu project. It's a Lua script that tells Lulu everything it needs to know: what your project is, what files it uses, what it depends on, and how to build it.

Let's break down all the important fields.

## `manifest`

**Type**: `table` | **Required**: `true`

This table holds all the metadata about your project. Think of it like the cover of a book. For now, it's mostly for informational purposes, but I might make Lulu do more with it in the future.

```lua
manifest = {
  name = "my-awesome-project", -- Required
  version = "1.0.0",
  description = "A really cool project that does amazing things.",
  authors = {"Your Name <you@example.com>"},
  tags = { "cli", "tool", "awesome" }
}
```

## `mods`

**Type**: `table` | **Required**: `true`

This is where you map friendly module names to your Lua files. This lets Lulu know which files are part of your project, so you can `require()` them easily.

```lua
mods = {
  -- The `main` module is special: it's what `lulu run` looks for
  main = "src/main.lua",

  -- You can name your other modules whatever you want
  utils = "src/utils.lua",
  engine = "src/engine/init.lua"
}
```

Now, from anywhere in your project, you can just `require("utils")` to get `src/utils.lua`. No more messy relative paths!

## `dependencies`

**Type**: `table` (of strings) | **Required**: `false`

A list of all the external libraries your project needs. Lulu can grab them from GitHub or any other URL.

```lua
dependencies = {
  -- From a GitHub repo (you can specify a branch or commit hash)
  "github:username/repo",
  "github:username/repo@branch",
  "github:username/repo#commit",

  -- From a direct URL to a library bundle
  "https://example.com/path/to/package.lulib"
}
```

Just listing a dependency here downloads it. To actually *use* it in your code, you also need to add it to the `include` field.

## `fetch`

**Type**: `string` or `table` | **Required**: `false`

This field is for when you're building a library for others to use. It tells Lulu what to give them when they add your project as a dependency.

- **`fetch = "code"`**: Tells Lulu to just clone your whole repository. This is for libraries that need to be built from source.

- **`fetch = { lulib = "..." }`**: Tells Lulu where to find a pre-built `.lulib` file.

```lua
-- Option 1: Let users build from the source code
fetch = "code"

-- Option 2: Point users to pre-built files
fetch = {
  lulib = "https://github.com/user/repo/releases/download/v1.0.0/package.lulib",
  
  -- You can also include platform-specific goodies like .so, .dll, or .dylib files
  include = {
    linux = {"https://.../package-linux.so"},
    windows = {"https://.../package-windows.dll"},
    macos = {"https://.../package-macos.dylib"}
  }
}
```

## `include`

**Type**: `table` (of strings) | **Required**: `false`

This is how you tell your project to actually load the code from the dependencies you've fetched.

```lua
include = {
  -- Include a dependency by its library name (from its own manifest)
  "@libname",

  -- You can also include a local .lulib file
  "./path/to/local/lib.lulib"
}
```

When you include `@libname`, Lulu looks for a `libname.lulib` file in your project's `.lib/lulib/` folder, which is where all the fetched dependencies live.

## `build`

**Type**: `function` | **Required**: `false`

This is where the magic happens! This function defines your project's build process. When you run `lulu build`, Lulu just runs this function.

```lua
build = function()
  -- Step 1: Grab all our dependencies
  resolve_dependencies()

  -- Step 2: Bundle our main module into a runnable program
  bundle_main("main.lua")
end
```

For a full list of all the cool helper functions you can use in here, check out the [Build Environment](./build-environment.md) reference.

## `macros`

**Type**: `string` or `table` | **Required**: `false`

This lets you define your own custom macros and share them with any other project that uses yours as a library.

Check out the [Custom Macros](../macros/custom-macros.md) page to learn how to become a macro wizard.