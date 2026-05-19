local class = require 'lib/middleclass'
local Rect = require 'lib/rect'
local Element = class('Element')

function Element:initialize(x,y,width,height)
    self.rect = Rect:new(x, y, width, height)
    self.on_click = nil
    self.handle_mouse_event = nil
    self.update = nil
end

function Element:set_on_click(callback)
    self.on_click = callback
end

function Element:set_handle_mouse_event(callback)
    self.handle_mouse_event = callback
end

function Element:set_on_update(callback)
    self.update = callback
end

function Element:get_rect()
    return self.rect
end

return Element