# `cfg!`

> Transforming Macro

The `cfg!` macro enables conditional compilation, allowing you to include or exclude blocks of code based on compile-time conditions. This is useful for handling platform-specific code or for enabling debugging features only in development builds.

## Conditional on Operating System (OS)

You can write code that only compiles for a specific operating system.

```lua
cfg! OS, {
  linux {
    print("This code will only exist on Linux.")
  },
  windows {
    print("This code will only exist on Windows.")
  },
  macos {
    print("This code will only exist on macOS.")
  }
}
```

Lulu automatically detects the target OS and only includes the matching code block in the final bundle.

## Conditional on Environment Variables

`cfg!` can also check for the existence or value of environment variables at compile time.

### Checking for Existence

Provide a second block to handle the case where the variable is not defined.

```lua
-- The `build` function in lulu.conf.lua can set env vars
-- set_env("DEBUG_MODE", true)

cfg! DEBUG_MODE, {
  print("Debug mode is enabled.")
}, {
  print("Debug mode is disabled.")
}
```

### Checking for a Specific Value

You can provide a block of branches to match against the value of the environment variable.

```lua
-- set_env("BUILD_TARGET", "PRODUCTION")

cfg! BUILD_TARGET, {
  PRODUCTION {
    print("Creating a production build.")
  },
  DEVELOPMENT {
    print("Creating a development build.")
  },
  -- A default case if no other branch matches
  _ {
    print("Unknown build target.")
  }
}
```

## Setting Compile-Time Variables

You can use `cfg! set` to define a variable that can be used by other `cfg!` blocks during the compilation process.

```lua
cfg! set, {
  MY_CUSTOM_FLAG = true
}

cfg! MY_CUSTOM_FLAG, {
  print("My custom flag was set!")
}
```
