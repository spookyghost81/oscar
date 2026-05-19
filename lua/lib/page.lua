local class = require 'lib/middleclass'
local Color = require 'lib/color'
local Element = require 'lib/element'
local Page = class('Page', Element)

function Page:initialize(x, y, width, height)
    Element.initialize(self, x or 0, y or 0, width or 800, height or 600)
    self.children = {}
    self.previous_mouse_state = nil
end

function Page:draw(ctx)
    for _, element in ipairs(self.children) do
        element:draw(ctx)
    end    
end

function Page:update(dt)
    for _, element in ipairs(self.children) do
        if element.update then
            element:update(dt)
        end
    end
end

function Page:update_mouse_state(mouse_state)
    local x = mouse_state.x
    local y = mouse_state.y
    local left_pressed = mouse_state.left_pressed
    local right_pressed = mouse_state.right_pressed
    local left_down = mouse_state.left_down
    local right_down = mouse_state.right_down

    -- print("Mouse event at: " .. tostring(x) .. ", " .. tostring(y) .. ", left_pressed: " .. tostring(left_pressed) .. ", right_pressed: " .. tostring(right_pressed))
    
    if left_pressed then
        self:handle_mouse_event({type = "primary_down", x = x, y = y})
    elseif left_down then
        self:handle_mouse_event({type = "primary_drag", x = x, y = y})
    elseif not left_down and self.previous_mouse_state and self.previous_mouse_state.left_down then
        self:handle_mouse_event({type = "primary_up", x = x, y = y})
    elseif right_pressed then
        self:handle_mouse_event({type = "secondary_down", x = x, y = y})
    elseif right_down then
        self:handle_mouse_event({type = "secondary_drag", x = x, y = y})
    elseif not right_down and self.previous_mouse_state and self.previous_mouse_state.right_down then
        self:handle_mouse_event({type = "secondary_up", x = x, y = y})
    else 
        self:handle_mouse_event({type = "move", x = x, y = y})
    end             
    
    self.previous_mouse_state = mouse_state
end

function Page:handle_mouse_event(event)
     for _, element in ipairs(self.children) do   
        local rect = element:get_rect()
        if rect:contains_point(event.x, event.y) and element.handle_mouse_event ~= nil then
            element:handle_mouse_event(event)
        end
    end
end

return Page