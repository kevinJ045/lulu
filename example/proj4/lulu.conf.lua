manifest = {
  name = "proj4",
  version = "0.1.0"
}

mods = {
  main = "main.lua"
}

build = function()
  resolve_dependencies()
  bundle_main("main.lua")

  print('Built binary to ".lib" folder.')
end
