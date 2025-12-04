


in local loader do

  function init(g)
    g.RAYOUS_MODULES = {}
  end

  local function lookup(g, path)
    if exists(path:join("route.lua")) then
      print(path, path:join("route.lua"))
      table.insert(g.RAYOUS_MODULES, { path, path:join("route.lua") })
    end
  end

  function loads(g, conf, root_path)
    local root = root_path:join(conf.pathing.path)

    lookup(g, root)
  end
end

return loader
