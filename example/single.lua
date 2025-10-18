
local items = range(0, 10, 5)

using(function(env)
  env.ss = "sss"
end)

print(ss)

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

fprint(lml! {
  <table id="mytable">
    {
      foreach(items)(function(id)
        return <item id={f"item-{id}"} style={make_style(id)} onClick={function() print("hi") end} />
      end)
    }
  </table>
})


local m = "sss"

print(match! m, {
  "ssss" {
    return "IT IS SSSS"
  }
  "not ss" {
    return "meh"
  }
  (val == "sss") {
    return "IT is sss"
  }
})

match! 1, {
  10 {
    print("IT IS SSS")
  }
  _ {
    print("meh")
  }
}

for_each! item, items, {
  cfg! OS, {
    linux {
      print(item)
    }
  }
}
