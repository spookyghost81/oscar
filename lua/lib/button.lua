local class = require 'lib/middleclass'
local Color = require 'lib/color'
local Element = require 'lib/element'

local Button = class('Button', Element)

function Button:initialize(x, y, width, height)
    Element.initialize(self, x, y, width, height)
    self.label = "Button"
    self.onClick = function() end
    self.x = x or 0
    self.y = y or 0
    self.width = width or 40
    self.height = height or 20
end

-- onClick can be either function or string (will be sent as toggle)
function Button:setOnClick(callback)
    self.onClick = callback
end

-- label can be either function or string
function Button:setLabel(label)
    self.label = label
end

function Button:draw(ctx)
    ctx:drawFilledRect(self.x, self.y, self.width, self.height, Color:new(100, 100, 100))
    local labelText = ""
    if type(self.label) == "function" then
        self.label(ctx)
    elseif type(self.label) == "string" then
        labelText = self.label
    end

    ctx:setFont("CossetteTexte-Regular", 24)
    ctx:setTextColor(Color.WHITE)
    ctx:setTextAlign("center", "center")

    ctx:drawText(labelText, self.x + self.width / 2, self.y + self.height / 2)
end

print("Button class loaded")

return Button