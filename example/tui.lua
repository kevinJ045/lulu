using {
  lulib.tui,
  lulib.threads
}

local app = tui.app()

local children = {
  tui.paragraph("lulu tui"),
  tui.barchart({{"a", 10}, {"b", 20}}),
  tui.layout({
    direction = "horizontal",
    constraints = { "50%", "50%" },
    children = {
      tui.list({ "Option A", "Option B", "Option C" }),
      tui.tabs({ "Home", "Stats", "About" }, 1)
    }
  })
}

async(function()
  sleep(4)
  children = {
    tui.paragraph("lulu tui"),
    tui.paragraph("lulu tui"),
    tui.paragraph("lulu tui"),
  }
end)

async(function()

  while app:is_open() do
    app:draw(tui.layout({
      direction = "vertical",
      constraints = { "20%", "20%", "60%" },
      children = children
    }))
    local k = app:poll()
    if k and k.type == "key" then
      if k.key == "q" then
        app:close()
      end
    end
    coroutine.yield()
  end
end)

