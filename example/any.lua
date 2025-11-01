using { lulib.threads }

local t = threads.spawn(function()
  threads.sleep(2)
end)

threads.join(t)