# UI Stub

A simple stub made for making simple UI capable binaries with lulu.

## Setup

To get started, you have to use the `lulu-ui` stub from [github](https://github.com/kevinj045/lulu-ui-stub/).

-   **First**, setup your environment. You might need to install some native libraries to make this work.
    
-   **Second**, add this to your [`build` function](../reference/build-environment.md).
    ```lua
    build = function()
      -- ...
      stubs {
        -- Windows binary
        windows = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui.exe",

        -- Linux x86_64 bin
        ["linux-x86_64"] = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui-linux-x86_64",

        -- Linux aarch64 bin (never tried this)
        ["linux-aarch64"] = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui-linux-aarch64",

        -- Darwin Executable (never tried this either)
        darwin = "https://github.com/kevinJ045/lulu-ui-stub/releases/download/v0.1.34/lulu-ui-darwin"
      }
      -- ...
    end
    ```

-   **Third**, Add a demo code into your `main.lua` as:
    ```lua
    local btn = ui.Button { text = "hi" }

    btn:into_root()
    ```

-   **Last**, Run the binary as:
    ```bash
      # -b for build
      lulu run -b
    ```
