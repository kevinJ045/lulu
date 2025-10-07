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

  include_bytes("")

  print('Built binary to ".lib" folder.')
end
