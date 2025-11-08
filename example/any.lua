using {
  lulib.tui,
  lulib.threads
}

local app = tui.app()


async(function()

  while app:is_open() do
    app:draw(tui.layout({
      direction = "vertical",
      constraints = { "100%" },
      children = {
        tui.table({
          "name", "id", "age", "personality"
        }, {
          {"someone", 1, 3, "dd"},
          {"onesome", 2, 5, "ddd"},
        })
      }
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

