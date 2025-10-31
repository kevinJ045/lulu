# The Build Environment

When you define a `build` function in your `lulu.conf.lua`, it is executed in a special environment that contains a set of helper functions for automating your build process.

```lua
-- lulu.conf.lua
build = function()
  -- You can use the helper functions here
  resolve_dependencies()
  bundle_main("main.lua")
end
```

Here are the functions available in that environment.

## Core Build Functions

- **`resolve_dependencies()`**: Resolves and fetches all dependencies listed in the `dependencies` field of your `lulu.conf.lua`. It populates the `.lib` directory with the fetched artifacts.

- **`download_file(url)`**: Downloads a URL into a cache path and returns the cache path.

- **`set_stub(path)`**: Set the [stub](../stubs/README.md) into a predetermined existing path.

- **`stubs({stub_map})`**: Set the [stub](../stubs/README.md) from a platform-specific URL map.

- **`bundle_main(entry_module, is_lib)`**: Bundles the project starting from the given entry module.
  - `entry_module` (string): The name of the module from your `mods` table to use as the entry point (e.g., `"main"`).
  - `is_lib` (boolean, optional): If `true`, creates a `.lulib` library bundle. If `false` or omitted, creates a standalone executable.
  - The output is placed in the `.lib/` directory, named after your project.

- **`bundle(entry_path, output_path)`**: A more direct bundling function.
  - `entry_path` (string): Path to the main Lua file to start bundling from.
  - `output_path` (string): Path for the output file. If it ends in `.lulib`, a library is created; otherwise, an executable is created.

- **`build(path)`**: Triggers a build for another Lulu project located at the given path.

## Environment and Configuration

- **`set_env(key, value)`**: Sets an environment variable for the duration of the build process. This can be used to conditionally control build steps in dependency projects.
  - `key` (string): The environment variable name.
  - `value` (string or boolean).

- **`set_cfg_env(key, value)`**: Similar to `set_env`, but sets a variable local to the current build instance only. It does not persist for sub-builds.

## File Operations

- **`include_bytes(name, path)`**: Includes the raw content of a file as bytes in the final bundle. This is useful for embedding assets like images or data files.
  - `name` (string): The name to assign to the byte asset.
  - `path` (string): The path to the file.

- **`exists(path)`**: Returns `true` if a file or directory exists at the given path, `false` otherwise.
