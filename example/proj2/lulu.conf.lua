-- lulu.conf.lua
manifest = {
  name = "proj2",
  version = "0.1.0",
}

fetch = {
	lulib = "http://localhost:3000/lib.lulib"
}

macros = io.open("_macros.lua"):read("*a")

mods = {
  init = "main.lua"
}
