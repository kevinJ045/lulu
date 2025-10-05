# Fetch Field Examples

This document shows how the Lulu package manager handles different `fetch` field configurations in GitHub repositories.

## Case 1: Fetch = "code" (Clone Repository)

If a GitHub repository has this in its `lulu.conf.lua`:

```lua
-- lulu.conf.lua
manifest = {
  name = "my-source-package",
  version = "1.0.0"
}

fetch = "code"  -- Indicates this package should be cloned

mods = {
  main = "main.lua",
  utils = "utils.lua"
}

build = function()
  bundle_main_exec("main.lua")
end
```

**Behavior**: The package manager will:
1. Clone the entire GitHub repository to cache
2. Execute the build function to create `.lulib` files
3. Copy the resulting artifacts to your project

**Command**: `lulu install github:user/my-source-package`

## Case 2: Fetch = lulib object (Download Pre-built)

If a GitHub repository has this in its `lulu.conf.lua`:

```lua
-- lulu.conf.lua
manifest = {
  name = "my-binary-package",
  version = "2.0.0"
}

fetch = {
  lulib = "https://github.com/user/my-binary-package/releases/download/v2.0.0/my-binary-package.lulib",
  include = {
    linux = {
      "https://github.com/user/my-binary-package/releases/download/v2.0.0/libmypackage-linux.so"
    },
    windows = {
      "https://github.com/user/my-binary-package/releases/download/v2.0.0/libmypackage-windows.dll"
    },
    macos = {
      "https://github.com/user/my-binary-package/releases/download/v2.0.0/libmypackage-macos.dylib"
    }
  }
}

-- No build function needed since we're downloading pre-built files
```

**Behavior**: The package manager will:
1. Download the `.lulib` file from the specified URL
2. Download platform-specific dynamic libraries for the current platform
3. Copy files directly to your project (no building required)

**Command**: `lulu install github:user/my-binary-package`

## Case 3: No Fetch Field (Default to Cloning)

If a GitHub repository has this in its `lulu.conf.lua`:

```lua
-- lulu.conf.lua
manifest = {
  name = "legacy-package",
  version = "0.5.0"
}

mods = {
  main = "main.lua"
}

build = function()
  bundle_main_exec("main.lua")
end

-- No fetch field specified
```

**Behavior**: The package manager will default to cloning behavior:
1. Clone the entire GitHub repository to cache
2. Execute the build function to create `.lulib` files
3. Copy the resulting artifacts to your project

**Command**: `lulu install github:user/legacy-package`

## Case 4: Repository Without lulu.conf.lua

If a GitHub repository doesn't have a `lulu.conf.lua` file:

**Behavior**: The package manager will:
1. Fall back to cloning the repository
2. Skip building (no configuration available)
3. The package will likely fail to install properly

**Recommendation**: Ensure all packages have proper `lulu.conf.lua` files.

## Debugging

To see what the package manager is doing:

```bash
# Install with verbose output
lulu install github:user/package-name

# Check what's cached
lulu cache list

# Clear cache for a package to force re-download
lulu cache remove github:user/package-name

# Clear all cache
lulu cache clear
```

## Best Practices for Package Authors

### For Source Packages (should be built from source):
```lua
fetch = "code"
```

### For Binary Packages (provide pre-built releases):
```lua
fetch = {
  lulib = "https://github.com/user/repo/releases/download/v1.0.0/package.lulib",
  include = {
    linux = {"https://github.com/user/repo/releases/download/v1.0.0/package-linux.so"},
    windows = {"https://github.com/user/repo/releases/download/v1.0.0/package-windows.dll"},
    macos = {"https://github.com/user/repo/releases/download/v1.0.0/package-macos.dylib"}
  }
}
```

This approach allows packages to choose the most appropriate distribution method for their use case.