-- lulu.conf.lua
manifest = {
  name = "proj2",
  version = "0.1.0",
}

fetch = {
	lulib = "http://localhost:3000/lib.lulib"
}

macros = require("_macros")

mods = {
  init = "main.lua"
}
