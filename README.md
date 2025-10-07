# Lulu
A simple Lua runtime that also helps with bundling Lua code, resolving Lua requirements, and building a final executable with libraries.

## Features
*   **Lua Runtime:** Execute Lua scripts.
*   **Code Bundling:** Bundle multiple Lua files into a single file.
*   **Dependency Resolution:** Resolve Lua module requirements.
*   **Executable Builder:** Create standalone executables from your Lua projects.

## Usage
### Running Lua files

You can run a single Lua file:
```bash
lulu run single.lua
```

You can also run a project with a `main.lua` and a `lulu.conf.lua`:

```bash
lulu run main.lua
```

### Bundling
Lulu can bundle your project into a single `.lulib` file:
```bash
lulu bundle main.lua /path/to/main.lulib
```

### Building an executable
You can build a standalone executable from your project. The build process is defined in your `lulu.conf.lua`.

```bash
lulu build /path/to/project
# or
lulu build # means lulu build .
```

### Dependency Management
Lulu can resolve and cache dependencies from URLs or git repositories.

To resolve all dependencies for a project, run `resolve` in the project directory:

```bash
lulu resolve .
```

You can also resolve a single dependency:
```bash
lulu resolve github:user/repo
```

You can manage the cache with the `cache` command:
```bash
# List all cached packages
lulu cache list

# Clear the entire cache
lulu cache clear

# Remove a specific package from the cache
lulu cache remove <package_url>
```

## TODO

*   [x] Runtime based on luajut
*   [x] A few simple APIs
*   [x] Lua bytecode buffer
*   [x] Lua module resolver from buffer
*   [x] Bundler into ".lulib" or executable
*   [x] Current path resolver
*   [x] Downloading lulib from paths
*   [x] Downloading lulib from github
*   [x] Building lua projects
*   [x] Building github repos
*   [x] Caches
*   [x] Macros
*   [x] Builtin Macros (like `import!`, `test!`, `cfg!`)
*   [x] Export Macros (through `lulu.conf.lua`)
*   [x] Basic Testing
*   [ ] Packager for Windows, Linux, and macOS
*   [ ] Testing framework integration
*   [ ] Additional features for FFI
*   [ ] LuaRocks integration

