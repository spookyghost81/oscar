local class = require 'lib/middleclass'

local Color = class('Color')

function Color:initialize(r, g, b, a)
    self.r = r or 255
    self.g = g or 255
    self.b = b or 255
    self.a = a or 255
end

Color.static.WHITE = Color:new(255, 255, 255, 255)
Color.static.BLACK = Color:new(0, 0, 0, 255)
Color.static.RED = Color:new(255, 0, 0, 255)
Color.static.GREEN = Color:new(0, 255, 0, 255)
Color.static.BLUE = Color:new(0, 0, 255, 255)
Color.static.TRANSPARENT = Color:new(0, 0, 0, 0)

return Color