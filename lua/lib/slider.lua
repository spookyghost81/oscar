local class = require 'lib/middleclass'
local Color = require 'lib/color'
local Element = require 'lib/element'

local Slider = class('Slider', Element)

function Slider:initialize(x, y, width, height, min_value, max_value)
    Element.initialize(self, x, y, width, height)
    self.value = 0.5
    self.min_value = min_value or 0
    self.max_value = max_value or 1
    self.on_change = function(value) end
end

function Slider:handle_mouse_event(event)
    -- print("Slider received mouse event: " .. tostring(event.type) .. " at (" .. event.x .. ", " .. event.y .. ")")
    if event.type == "primary_down" or event.type == "primary_drag" then
        local rect = self:get_rect()
        local local_y = rect:world_to_local_y(event.y)
        local y_frac = 1 - local_y
        local value = self.min_value + y_frac * (self.max_value - self.min_value)
        -- self:update_value(value)
        self:on_change(value)
    end 
end

function Slider:set_on_change(callback)
    self.on_change = callback
end

function Slider:update_value(value)
    --print("Slider value updated to: " .. value)
    self.value = math.max(self.min_value, math.min(self.max_value, value))
    -- self:on_change(self.value)
end

function Slider:draw(ctx)
    local rect = self:get_rect()
    local width = rect:get_width()
    local height = rect:get_height()
    ctx:draw_outlined_rect(rect:get_left(), rect:get_top(), width, height, Color:new(50, 50, 50))
    local y_frac = (self.value - self.min_value) / (self.max_value - self.min_value)
    local handle_y = rect:get_top() + (1 - y_frac) * height
    ctx:draw_filled_rect(rect:get_left(), handle_y - 5, width, 10, Color:new(200, 200, 200))
end

return Slider