# Lulu Package Manager

The Lulu runtime now includes a comprehensive package manager that can handle code by cloning GitHub repositories and/or downloading .zip/.tar.gz archives. Packages are cached in `~/.cache/lulu` (Linux/macOS) or `%APPDATA%/lulu` (Windows) and built there before copying artifacts to your project.

## Features

- **Multiple Source Support**: Clone from GitHub repositories, download ZIP/tar.gz archives, or clone Git repositories
- **Intelligent Caching**: Packages are cached locally to speed up subsequent installations
- **Cross-platform**: Works on Linux, macOS, and Windows with appropriate dynamic library handling
- **Automatic Building**: Packages are automatically built in the cache directory
- **Artifact Management**: Automatically copies `.lulib` files and platform-specific dynamic libraries

## Commands

### Install Packages

```bash
# Install packages from various sources
lulu install github:user/repo
lulu install https://github.com/user/repo.git
lulu install https://example.com/package.zip
lulu install https://example.com/package.tar.gz

# Install multiple packages
lulu install github:user/repo1 github:user/repo2 https://example.com/package.zip

# Install to a specific project directory
lulu install --project /path/to/project github:user/repo
```

### Update Packages

```bash
# Update packages (clears cache first, then reinstalls)
lulu update github:user/repo
lulu update github:user/repo1 github:user/repo2
```

### Cache Management

```bash
# List cached packages
lulu cache list

# Clear all cache
lulu cache clear

# Remove specific package from cache
lulu cache remove github:user/repo
```

### Resolve Dependencies

```bash
# Resolve a single dependency
lulu resolve github:user/repo
lulu resolve https://example.com/package.zip

# Resolve dependencies for a project (reads lulu.conf.lua)
lulu resolve /path/to/project
```

## Package Sources

### GitHub Repositories

```bash
# Basic GitHub repository
lulu install github:username/repository

# Specific branch
lulu install github:username/repository@branch-name

# Specific commit
lulu install github:username/repository#commit-hash

# Subdirectory within repository
lulu install github:username/repository/subdirectory

# Combined: subdirectory with specific branch
lulu install github:username/repository/subdirectory@branch-name
```

### Git Repositories

```bash
# Any Git repository
lulu install https://github.com/user/repo.git
lulu install https://gitlab.com/user/repo.git
```

### Archives

```bash
# ZIP files
lulu install https://example.com/package.zip

# Tar.gz files
lulu install https://example.com/package.tar.gz
lulu install https://example.com/package.tgz
```

## How It Works

### For GitHub Repositories (`github:user/repo`)
1. **Check Fetch Field**: First downloads the `lulu.conf.lua` from the GitHub repository
2. **Handle Based on Fetch Field**:
   - If `fetch = "code"`: Clones the repository to cache
   - If `fetch = { lulib = "url", include = {...} }`: Downloads the `.lulib` file and platform-specific libraries
   - If no fetch field: Defaults to cloning the repository
3. **Build**: Executes the package's build process in the cache directory (only for cloned repositories)
4. **Copy**: Copies built artifacts to your project:
   - `.lulib` files go to `.lib/lulib/`
   - Dynamic libraries go to `.lib/dylib/` (platform-specific: `.so`, `.dll`, `.dylib`)

### For Other Sources (URLs, Archives)
1. **Fetch**: Downloads or extracts the package to a cache directory using a SHA-256 hash of the URL
2. **Build**: Executes the package's build process in the cache directory (if it has a `lulu.conf.lua`)
3. **Copy**: Copies built artifacts to your project

## Cache Directory Structure

```
~/.cache/lulu/  (or %APPDATA%/lulu on Windows)
├── abc123def456/          # Package cache (hashed directory name)
│   ├── lulu.conf.lua     # Package configuration
│   ├── main.lua          # Package source files
│   └── .lib/             # Built artifacts
│       ├── lulib/
│       │   └── mylib.lulib
│       └── dylib/
│           └── mylib.so
└── fed789cba321/          # Another package cache
    └── ...
```

## Project Structure

After installing packages, your project will have:

```
your-project/
├── lulu.conf.lua
├── main.lua
└── .lib/
    ├── lulib/            # Copied .lulib files
    │   ├── package1.lulib
    │   └── package2.lulib
    └── dylib/            # Copied dynamic libraries
        ├── package1.so   # Linux
        ├── package1.dll  # Windows
        └── package1.dylib # macOS
```

## Package Configuration

Packages should include a `lulu.conf.lua` file for proper building and metadata:

```lua
-- lulu.conf.lua
manifest = {
  name = "mypackage",
  version = "1.0.0",
  readme = "README.md",
  tags = {"lua", "lulu", "library"}
}

mods = {
  main = "main.lua",
  utils = "utils.lua"
}

-- For packages that should be cloned:
fetch = "code"

-- OR for packages that provide pre-built .lulib files:
fetch = {
  lulib = "https://github.com/user/repo/releases/download/v1.0.0/mypackage.lulib",
  include = {
    linux = {"https://github.com/user/repo/releases/download/v1.0.0/mypackage-linux.so"},
    windows = {"https://github.com/user/repo/releases/download/v1.0.0/mypackage-windows.dll"},
    macos = {"https://github.com/user/repo/releases/download/v1.0.0/mypackage-macos.dylib"}
  }
}

build = function()
  -- Build process here (only runs if fetch = "code")
  bundle_main_exec("main.lua")
end
```

## Error Handling

- If a package fails to install, other packages in the same command continue to install
- Build failures are reported with full output for debugging
- Network failures are handled gracefully with informative error messages
- Missing dependencies (like Git) are reported clearly

## Platform-Specific Notes

### Linux
- Dynamic libraries use `.so` extension
- Cache stored in `~/.cache/lulu/`
- Requires `git` command for Git operations

### macOS
- Dynamic libraries use `.dylib` extension
- Cache stored in `~/.cache/lulu/`
- Requires `git` command for Git operations

### Windows
- Dynamic libraries use `.dll` extension
- Cache stored in `%APPDATA%/lulu/`
- Requires `git.exe` in PATH for Git operations

## Integration with lulu.conf.lua

You can use the package manager within your project's build scripts:

```lua
-- In your lulu.conf.lua
dependencies = {
  "github:user/some-library",
  "https://example.com/other-lib.zip"
}

build = function()
  if not exists(".lib/lulib/some-library.lulib") then
    resolve_dependencies()
  end
  
  bundle_main_exec("main.lua")
end
```

## Examples

### Installing a GitHub Package

```bash
# Install a Lua library from GitHub
lulu install github:kevinj045/lua-utils

# Install from a specific subdirectory
lulu install github:kevinj045/monorepo/lua-utils

# Install a specific version
lulu install github:kevinj045/lua-utils@v1.2.0
```

### Installing from Archives

```bash
# Install from a ZIP file
lulu install https://github.com/user/repo/archive/main.zip

# Install from a tar.gz release
lulu install https://github.com/user/repo/releases/download/v1.0.0/package.tar.gz
```

### Managing Dependencies for a Project

```bash
# Navigate to your project directory
cd my-lulu-project

# Install dependencies listed in lulu.conf.lua
lulu resolve .

# Or specify the project path
lulu resolve /path/to/my-lulu-project
```