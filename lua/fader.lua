local class = require 'lib/middleclass'

local Slider = require 'lib/slider'
local Color = require 'lib/color'

local Fader = class('Fader', Slider)

function Fader:initialize(x, y, width, height, min, max)
    Slider.initialize(self, x, y, width, height, min, max)
    self.vu_lr = 0
    self.vu_l = 0
    self.vu_r = 0
end

function Fader:update_vu(lr, l, r)
    self.vu_lr = lr
    self.vu_l = l
    self.vu_r = r
end

function Fader:draw(ctx)
    local rect = self:get_rect()
    local columns = rect:divide_horizontal(3)
    local vu_l_rect = columns[1];
    local vu_lr_rect = columns[2];
    local vu_r_rect = columns[3];

    local vu_l_height = vu_l_rect:get_height() * self.vu_l
    local vu_lr_height = vu_lr_rect:get_height() * self.vu_lr
    local vu_r_height = vu_r_rect:get_height() * self.vu_r

    local vu_l_rect = vu_l_rect:remove_from_bottom(vu_l_height)
    local vu_lr_rect = vu_lr_rect:remove_from_bottom(vu_lr_height)
    local vu_r_rect = vu_r_rect:remove_from_bottom(vu_r_height)
    
    local vu_l_color = Color:new(0, 255, 0)
    if self.vu_l > 0.9 then vu_l_color = Color:new(255, 0, 0) end
    local vu_lr_color = Color:new(0, 255, 0)
    if self.vu_lr > 0.9 then vu_lr_color = Color:new(255, 0, 0) end
    local vu_r_color = Color:new(0, 255, 0)
    if self.vu_r > 0.9 then vu_r_color = Color:new(255, 0, 0) end


    ctx:draw_filled_rect(vu_l_rect:get_left(), vu_l_rect:get_top(), vu_l_rect:get_width(), vu_l_rect:get_height(), vu_l_color)
    ctx:draw_filled_rect(vu_lr_rect:get_left(), vu_lr_rect:get_top(), vu_lr_rect:get_width(), vu_lr_rect:get_height(), vu_lr_color)
    ctx:draw_filled_rect(vu_r_rect:get_left(), vu_r_rect:get_top(), vu_r_rect:get_width(), vu_r_rect:get_height(), vu_r_color)

    ctx:set_text_align("center", "center")
    ctx:draw_text(string.format("%.2f", self.vu_l), self.rect:local_to_world_x(0.5), self.rect:local_to_world_y(0.25), Color.WHITE)
    Slider.draw(self, ctx)
end

return Fader