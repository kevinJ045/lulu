
local items = range(0, 10, 5)

function lml_create(name, props, ...)
  return {
    name = name,
    props = props,
    children = ...
  }
end

cfg! OS_LINUX, {
  print("linux")
}

cfg! OS_WINDOWS, {
  print("windows")
}

print(cfg! OS, {
  linux {
    "linux"
  }
  window {
    "windows"
  }
})

cfg! set, {
  MY_VAR = SOMETHING
}

cfg! MY_VAR, {
  print("SOMETHING IS DEFINED")
}

cfg! MY_ENV_VAR, {
  print("env var defined")
}, {
  print("env var undefined")
}

local m = {
  id = "ss"
}
local mp = &m
local mv = *mp
print(mv)

function make_style(id)
  return {
    name = f"ss {{something}} {id}"
  }
end

iprint(lml! {
  <table id="mytable">
    {
      foreach(items)(function(id)
        return <item id={f"item-{id}"} style={make_style(id)} onClick={function() print("hi") end} />
      end)
    }
  </table>
})

for_each! item, items, {
  cfg! OS, {
    linux {
      print(item)
    }
  }
}