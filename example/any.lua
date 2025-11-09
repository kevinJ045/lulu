using {
  lulib.minifb
}

local bytes = read("../assets/logo.png")

local win = minifb.window("hii", 800, 600, {
  resize = true
})

local img = win:load_image(bytes)

x, y = 0, 0
up, left = false, false
speed = 10

win:handle(function(u)
  if u == "update" then
    win:clear(0x111111)
    win:fill_rect(x, y, 100, 100, 0xff0000)
    win:draw_image(x, y, img, 100, 100)
    if up then
      y -= speed
    else
      y += speed
    end

    if left then
      x -= speed
    else
      x += speed
    end
  
    if y < 0 then
      y = 0
      up = false
    elseif y + 100 > win.height then
      up = true
    end
  
    if x < 0 then
      x = 0
      left = false
    elseif x + 100 > win.width then
      left = true
    end
  end
end)

win:start()