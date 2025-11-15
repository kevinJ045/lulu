# Dylib

You can use `dylib` to include dynamic libraries.

```lua
using {
  -- will look for the dylib
  -- inside of the possible 
  -- paths
  dylib"@dylib_name""mylib"[[
    int add(int a, int b);
  ]]
  -- or separate cdef
  dylib_cdef[[
    int add(int a, int b);
  ]]
}

mylib.add(1, 2) -- should work
```