-- lulu.conf.lua
manifest = {
  name = "proj3",
  version = "0.1.0",
}

fetch = "code"

mods = {
  init = "main.lua"
}

include = {
  "@lib"
}

dependencies = {
  "github:kevinJ045/demo-lulu-proj"
}

build = function()
  print("Resolving deps")

  resolve_dependencies()
  
  bundle_main("main.lua")
end

