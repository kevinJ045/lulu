
using {
  lulib.pathing
}

local path = pathing.appdata():join("my-app"):ensure()
print(path)
-- path.extension.ensure()