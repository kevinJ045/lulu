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
  "@proj3"
}

dependencies = {
  "github:kevinj045/demo-lulu-proj-2"
}

build = function()
  if not exists(".lib/lulib/lib.lulib") then
    resolve_dependencies()
  end
  
  bundle_main_exec("main.lua")
end

