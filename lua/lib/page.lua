local class = require 'lib/middleclass'
local Color = require 'lib/color'
local Page = class('Page')

function Page:initialize(width, height)
    self.width = width or 800
    self.height = height or 600
    self.elements = {}
    self.previous_mouse_state = nil
end

function Page:draw(ctx)
    ctx:draw_filled_rect(0, 0, self.width, self.height, Color.BLACK)
    for _, element in ipairs(self.elements) do
        element:draw(ctx)
    end

    --draw prev mouse state
    ctx:set_text_align("left", "bottom");
    ctx:draw_text("Mouse State: " .. (self.previous_mouse_state and ("x: " .. self.previous_mouse_state.x .. ", y: " .. self.previous_mouse_state.y .. ", left_pressed: " .. tostring(self.previous_mouse_state.left_pressed) .. ", right_pressed: " .. tostring(self.previous_mouse_state.right_pressed)) or "nil"), 10, self.height - 30)
end

function Page:update(dt)
    for _, element in ipairs(self.elements) do
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
    for _, element in ipairs(self.elements) do      
        local rect = element:get_rect()
        if rect:contains_point(x, y) and element.handle_mouse_event ~= nil then
            if left_pressed then
                element:handle_mouse_event({type = "primary_down", x = x, y = y})
            elseif left_down then
                element:handle_mouse_event({type = "primary_drag", x = x, y = y})
            elseif not left_down and self.previous_mouse_state and self.previous_mouse_state.left_down then
                element:handle_mouse_event({type = "primary_up", x = x, y = y})
            elseif right_pressed then
                element:handle_mouse_event({type = "secondary_down", x = x, y = y})
            elseif right_down then
                element:handle_mouse_event({type = "secondary_drag", x = x, y = y})
            elseif not right_down and self.previous_mouse_state and self.previous_mouse_state.right_down then
                element:handle_mouse_event({type = "secondary_up", x = x, y = y})
            else 
                element:handle_mouse_event({type = "move", x = x, y = y})
            end             
        end
    end

    self.previous_mouse_state = mouse_state
end

return Page