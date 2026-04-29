local class = require 'lib/middleclass'
local Element = class('Element')

function Element:initialize(x,y,width,height)
    self.x = x or 0
    self.y = y or 0
    self.width = width or 10
    self.height = height or 10
end

return Element