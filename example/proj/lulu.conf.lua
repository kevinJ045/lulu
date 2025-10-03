-- lulu.conf.lua
manifest = {
  name = "myproj",
  version = "0.1.0",
  readme = "README.md",
  tags = {"lua", "lulu", "runtime"}
}

mods = {
  main = "main.lua",
  utils = "utils.lua"
}

include = {
  "@lib"
}

dependencies = {
  "http://localhost:3000/lib.lulib"
}

build = function()
  if not exists(".lib/lulib/lib.lulib") then
    resolve_dependencies()
  end
  
  bundle_main_exec("main.lua")
end

