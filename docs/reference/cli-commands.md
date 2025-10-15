# CLI Commands

Lulu has a simple cli entry, which is `lulu` command. Here are most of the functions(if i hadn't forgotten anything):

## `lulu new`

Creates a new Lulu project directory.

```bash
lulu new <project-name> [options]
```

- **`<project-name>`**: The name of the project directory to create.

### Options

- `-g`, `--git`: Initializes a new Git repository in the project directory.
- `-i`, `--ignore`: Skips any interactive prompts and uses default values.
- `-t`, `--lib`: Initializes the project as a library, which may affect the default `lulu.conf.lua` structure.

## `lulu run`

Executes a Lua script or a Lulu project.

```bash
# Run the 'main' module of a project
lulu run main.lua

# Run a specific Lua file
lulu run path/to/script.lua

# Run a project and pass arguments to the script
lulu run main.lua -- arg1 arg2 "some value"
```

## `lulu build`

Builds the current project by executing the `build` function within `lulu.conf.lua`.

```bash
lulu build
```

This command is the standard way to build your project and produce artifacts like executables or library bundles. See the [Build Environment](./build-environment.md) page for details on what you can do inside the `build` function.

## `lulu bundle`

A direct command to bundle a set of Lua files into a single artifact without needing a full project setup.

```bash
# Create an executable from an entrypoint file
lulu bundle <entry-file.lua> <output-executable-path>

# Create a library bundle from an entrypoint file
lulu bundle <entry-file.lua> <output-library.lulib>
```

Lulu knows to create a library bundle if the output path ends with the `.lulib` extension.

## `lulu test`

Runs tests defined within a Lua file using the `test!` macro.

```bash
# Run all tests in a file
lulu test path/to/test_file.lua

# Run only a specific test block within the file
lulu test path/to/test_file.lua -t <test_name>
```

For more details, see the [Testing](./testing.md) reference page.

## `lulu cache`

Manages the cache where Lulu stores downloaded dependencies.

```bash
# List all cached packages
lulu cache list

# Remove a specific package from the cache
lulu cache remove <cache-key>

# Clear the entire cache
lulu cache clear
```
