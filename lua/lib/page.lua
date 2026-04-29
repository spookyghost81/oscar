local class = require 'lib/middleclass'

local Color = require 'lib/color'

local Page = class('Page')

function Page:initialize(width, height)
    self.width = width or 800
    self.height = height or 600
    self.elements = {}
end

function Page:draw(ctx)
    ctx:drawFilledRect(0, 0, self.width, self.height, Color.BLACK)
    for _, element in ipairs(self.elements) do
        element:draw(ctx)
    end
end

return Page