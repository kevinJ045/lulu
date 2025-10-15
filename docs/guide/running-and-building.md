# Running and Building

Lulu provides a few commands to run your code and/or bundle it for distribution.

## Running Lua Files

There are two primary ways to run code with Lulu.

### Running a Project

When inside a directory with a `lulu.conf.lua` file, the `lulu run` command will execute the project's `main` module.

```bash
# This will execute the file specified by the `main` key in `mods`
lulu run

# You can also pass arguments to your script
lulu run -- arg1 arg2
```

### Running a Single File

You can also run any `.lua` file directly, even outside of a project.

```bash
lulu run path/to/your/script.lua
```

## Bundling Your Code

"Bundling" is the process of collecting all your Lua modules into a single file. Lulu can create two types of bundles:

- A **`.lulib` library bundle**: A single Lua file containing all your modules, which can be required by other Lulu projects.
- A **standalone executable**: A single binary file that includes your bundled code and the Lua runtime itself.

### Bundling with `lulu bundle`

The `lulu bundle` command is a direct way to create a bundle from a specific entry point.

```bash
# Bundle main.lua and all its local dependencies into an executable
lulu bundle ./main.lua path/to/my_executable # .exe on windows

# Bundle init.lua into a library file
# The .lulib extension is important!
lulu bundle ./init.lua path/to/my_library.lulib
```

### Building a Project with `lulu build`

For projects, the `lulu build` command provides more utility and customization overall.

When you run `lulu build`, Lulu looks for a `build` function in your `lulu.conf.lua` file and executes it. This lets you define more complex build steps.

A common build process looks like this:

```lua
-- lulu.conf.lua

build = function()
  -- This helper function bundles the `main` module into an executable
  bundle_main("main.lua")

  -- Or, to create a library bundle instead:
  -- bundle_main("main.lua", true)
end
```

With the config above, running `lulu build` in will create an executable file in the `.lib/` directory of your project.

To learn more about creating powerful, custom build steps, see the [Build Environment reference](../reference/build-environment.md).
