
async(function()
  print("first")
end)

local frames = 0
function fps()
  async(function()
    sleep(1)
    print(f"{frames} / s")
    frames = 0
    fps()
  end)
end

async(function()
  print("last")
end)

print("normal")

async(function()
  while true do
    frames = frames + 1
    coroutine.yield()
  end
end)

fps()