local class = require 'lib/middleclass'

local Rect = class('Rect')

function Rect:initialize(x, y, width, height)
    self.x = x or 0
    self.y = y or 0
    self.width = width or 10
    self.height = height or 10
end

function Rect:set_position(x, y)
    self.x = x
    self.y = y
end

function Rect:set_size(width, height)
    self.width = width
    self.height = height
end

function Rect:get_left()
    return self.x
end

function Rect:get_right()
    return self.x + self.width
end

function Rect:get_top()
    return self.y
end

function Rect:get_bottom()
    return self.y + self.height
end

function Rect:get_width()
    return self.width
end

function Rect:get_height()
    return self.height
end

function Rect:remove_from_top(amount)
    local new_rect = Rect:new(self.x, self.y, self.width, amount)
    self.y = self.y + amount
    self.height = self.height - amount
    return new_rect
end

function Rect:remove_from_bottom(amount)
    local new_rect = Rect:new(self.x, self.y + self.height - amount, self.width, amount)
    self.height = self.height - amount
    return new_rect
end

function Rect:remove_from_left(amount)
    local new_rect = Rect:new(self.x, self.y, amount, self.height)
    self.x = self.x + amount
    self.width = self.width - amount
    return new_rect
end

function Rect:remove_from_right(amount)
    local new_rect = Rect:new(self.x + self.width - amount, self.y, amount, self.height)
    self.width = self.width - amount
    return new_rect
end

function Rect:divide_horizontal(columns)
    local column_width = self.width / columns
    local rects = {}
    for i = 0, columns - 1 do
        table.insert(rects, Rect:new(self.x + i * column_width, self.y, column_width, self.height))
    end
    return rects
end

function Rect:divide_vertical(rows)
    local row_height = self.height / rows
    local rects = {}
    for i = 0, rows - 1 do
        table.insert(rects, Rect:new(self.x, self.y + i * row_height, self.width, row_height))
    end
    return rects
end

function Rect:divide_grid(columns, rows)
    local column_width = self.width / columns
    local row_height = self.height / rows
    local rects = {}
    for i = 1, columns do
        rects[i] = {}
        for j = 1, rows do
            table.insert(rects[i], Rect:new(self.x + (i - 1) * column_width, self.y + (j - 1) * row_height, column_width, row_height))
        end
    end
    return rects
end

function Rect:contains_point(x,y)
    return x >= self:get_left() and x <= self:get_right() and
           y >= self:get_top() and y <= self:get_bottom()
end

function Rect:world_to_local(x, y)
    return {self:world_to_local_x(x), self:world_to_local_y(y)}
end

function Rect:world_to_local_x(x)
    return (x - self:get_left()) / self:get_width()
end

function Rect:world_to_local_y(y)
    return (y - self:get_top()) / self:get_height()
end

function Rect:local_to_world(x, y)
    return {self:local_to_world_x(x), self:local_to_world_y(y)}
end

function Rect:local_to_world_x(local_x)
    return self:get_left() + local_x * self:get_width()
end

function Rect:local_to_world_y(local_y)
    return self:get_top() + local_y * self:get_height()
end

function Rect:contracted(amount)
    local new_x = self.x + amount
    local new_y = self.y + amount
    local new_width = self.width - 2 * amount
    local new_height = self.height - 2 * amount
    return Rect:new(new_x, new_y, new_width, new_height)
end

function Rect:expanded(amount)
    local new_x = self.x - amount
    local new_y = self.y - amount
    local new_width = self.width + 2 * amount
    local new_height = self.height + 2 * amount
    return Rect:new(new_x, new_y, new_width, new_height)
end

return Rect