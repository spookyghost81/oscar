local class = require 'lib/middleclass'
local Color = require 'lib/color'
local Element = require 'lib/element'

local Button = class('Button', Element)

function Button:initialize(x, y, width, height)
    print("Button init")
    Element.initialize(self, x, y, width, height)
    self.label = "Button"
    self.x = x or 0
    self.y = y or 0
    self.width = width or 40
    self.height = height or 20
    self.bg_color = Color:new(100, 100, 100)
end

-- label can be either string or a function that returns a string
function Button:set_label(label)
    self.label = label
end

function Button:draw(ctx)
    ctx:draw_filled_rect(self.x, self.y, self.width, self.height, self.bg_color)
    local label_text = ""
    if type(self.label) == "function" then
        self.label(ctx)
    elseif type(self.label) == "string" then
        label_text = self.label
    end

    ctx:set_font("CossetteTexte-Regular", 24)
    ctx:set_text_color(Color.WHITE)
    ctx:set_text_align("center", "center")

    ctx:draw_text(label_text, self.x + self.width / 2, self.y + self.height / 2)
end

function Button:set_bg_color(color)
    self.bg_color = color
end

function Button:handle_mouse_event(event)
    --print("Button received mouse event: " .. tostring(event.type) .. " at (" .. event.x .. ", " .. event.y .. ")")
    if event.type == "primary_down" then
        local rect = self:get_rect()
        if rect:contains_point(event.x, event.y) then
            if self.on_click then
                self.on_click(self, event.x, event.y)
            end
        end
    end
end

return Button