using {
  lulib.minifb
}

local win = minifb.window("some", 200, 300)

win:draw_line(10, 10, 100, 50, 0xFF0000)
win:draw_circle(50, 50, 20, 0x00FF00)
win:draw_poly({{10,10}, {50,80}, {90,10}}, 0x0000FF)
win:update()