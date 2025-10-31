
# Stubs

A "stub" is basically a binary where your lulib will be appended into, not only changing the way your lulib runs, but also giving you an entirely new environment on top of Lulu. While Lulu uses itself by default, these binaries/stubs are interchangebale with the `stubs` function in your build environment. 

```lua
-- lulu.conf.lua
build = function()
  -- generally
  stubs {
    windows = "https://.../stub-windows.exe",
    ["linux-aarch64"] = "https://.../stub-linux-aarch64",
    linux = "https://.../stub-linux",
  }
  -- or
  if CURRENT_OS == "linux" then
    set_stub("path/to/stub")
  else
    ...
  end
  
  bundle_main("main.lua")
  -- the main result will be based on the stub provided
  -- the stub. the size of the final result is also
  -- based on the stub
end
```