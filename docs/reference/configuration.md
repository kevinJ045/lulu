# Configuration (`lulu.conf.lua`)

The `lulu.conf.lua` file is the heart of every Lulu project. It's a Lua script that acts as a manifest, defining your project's metadata, files, dependencies, and build process.

Here is a breakdown of all the major fields available.

## `manifest`

**Type**: `table` | **Required**: `true`

Contains metadata about your project. While most fields are for informational purposes now, they may be used by future tooling or package managers. (I may make lulu does more with these in the future).

```lua
manifest = {
  name = "my-project", -- Required
  version = "1.0.0",
  description = "A brief description of my project.",
  authors = {"Your Name <you@example.com>"},
  tags = { "cli", "tool" }
}
```

## `mods`

**Type**: `table` | **Required**: `true`

Maps a module name to a Lua file path. This tells Lulu which files are part of your project, allowing them to be bundled and required by their mapped name.

```lua
mods = {
  -- The `main` module is special; it's the entry point for `lulu run`
  main = "src/main.lua",

  -- Other modules can be named anything
  utils = "src/utils.lua",
  engine = "src/engine/init.lua"
}
```

Within your project, you can then use `require("utils")` to load `src/utils.lua`, otherwise it would be `require("project_name/utils")`.

## `dependencies`

**Type**: `table` (array of strings) | **Required**: `false`

Defines a list of external libraries your project depends on. Lulu can fetch dependencies from GitHub or direct URLs.

```lua
dependencies = {
  -- Fetch from a GitHub repository
  "github:username/repo",
  "github:username/repo@branch",
  "github:username/repo#commit",

  -- Fetch a library bundle from a URL
  "https://example.com/path/to/package.lulib"
}
```

When a dependency is fetched, Lulu downloads it to a central cache. To actually use the code from a dependency in your project, you must list it in the `include` field.

## `fetch`

**Type**: `string` or `table` | **Required**: `false`

This field is used when your project is intended to be used as a library by others. It tells Lulu what to provide when another project lists yours as a dependency.

- **`fetch = "code"`**: Tells Lulu to clone the entire repository. This is for libraries that need to be built from source.

- **`fetch = { lulib = "..." }`**: Tells Lulu to download a pre-built `.lulib` file from the specified URL.

```lua
-- Option 1: Build from source
fetch = "code"

-- Option 2: Download pre-built artifacts
fetch = {
  lulib = "https://github.com/user/repo/releases/download/v1.0.0/package.lulib",
  
  -- Optional: Include platform-specific dynamic libraries (.so, .dll, .dylib)
  include = {
    linux = {"https://.../package-linux.so"},
    windows = {"https://.../package-windows.dll"},
    macos = {"https://.../package-macos.dylib"}
  }
}
```

## `include`

**Type**: `table` (array of strings) | **Required**: `false`

Specifies which fetched dependencies should be made available to your project's runtime. This is how you gain access to the code you defined in the `dependencies` table.

```lua
include = {
  -- Include a dependency by its library name (from its manifest)
  "@libname",

  -- You can also include a .lulib file by its path
  "./path/to/local/lib.lulib"
}
```

When you include `@libname`, Lulu looks for `libname.lulib` inside your project's local `.lib/lulib/` directory, which is where dependencies are placed after being fetched.

## `build`

**Type**: `function` | **Required**: `false`

Defines the build process for your project. When you run `lulu build`, Lulu executes this function. A special environment with helper functions is available inside this function.

```lua
build = function()
  -- 1. Resolve and fetch all dependencies
  resolve_dependencies()

  -- 2. Bundle the main module into a standalone executable
  bundle_main("main.lua")
end
```

For a full list of available helper functions, see the [Build Environment](./build-environment.md) reference.

## `macros`

**Type**: `string` or `table` | **Required**: `false`

Lets you define and export custom macros from your project, making them available to any other project that uses yours as a library.

See the [Custom Macros](../macros/custom-macros.md) page for a detailed guide.
