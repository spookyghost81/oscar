local class = require 'lib/middleclass'

local Element = require 'lib/element'
local Button = require 'lib/button'
local Color = require 'lib/color'
local Page = require 'lib/page'
local Slider = require 'lib/slider'
local Rect = require 'lib/rect'

local DefaultPage = class('DefaultPage', Page)

function DefaultPage:initialize(width, height)
    Page.initialize(self, width, height)
    
    local rect = Rect:new(0, 0, self.width, self.height)
    local left_rect = rect:remove_from_left(self.width * 0.6)

    local columns = left_rect:divide_horizontal(8)
    for i, column in ipairs(columns) do
        local strip_rect = column
        local rec_button_rect = strip_rect:remove_from_top(30):contracted(5)
        local rec_button = Button:new(rec_button_rect:get_left(), rec_button_rect:get_top(), rec_button_rect:get_width(), rec_button_rect:get_height())
        rec_button:set_label("REC")
        rec_button:set_on_click(function(self, x, y)
            print("Track " .. i .. " REC button clicked")
        end)

        table.insert(self.elements, rec_button)

        local solo_button_rect = strip_rect:remove_from_top(30):contracted(5)
        local solo_button = Button:new(solo_button_rect:get_left(), solo_button_rect:get_top(), solo_button_rect:get_width(), solo_button_rect:get_height())
        solo_button:set_label("S")
        solo_button:set_on_click(function(self, x, y)
            print("Track " .. i .. " SOLO button clicked")
        end)

        table.insert(self.elements, solo_button)

        local mute_button_rect = strip_rect:remove_from_top(30):contracted(5)
        local mute_button = Button:new(mute_button_rect:get_left(), mute_button_rect:get_top(), mute_button_rect:get_width(), mute_button_rect:get_height())
        mute_button:set_label("M")
        mute_button:set_on_click(function(self, x, y)
            print("Track " .. i .. " MUTE button clicked")
        end)

        table.insert(self.elements, mute_button)

        local sel_button_rect = strip_rect:remove_from_top(30):contracted(5)
        local sel_button = Button:new(sel_button_rect:get_left(), sel_button_rect:get_top(), sel_button_rect:get_width(), sel_button_rect:get_height())
        sel_button:set_label("SEL")
        sel_button:set_on_click(function(self, x, y)
            print("Track " .. i .. " SEL button clicked")
        end)

        table.insert(self.elements, sel_button)

        local volume_slider_rect = strip_rect:contracted(5)
        local volume_slider = Slider:new(volume_slider_rect:get_left(), volume_slider_rect:get_top(), volume_slider_rect:get_width(), volume_slider_rect:get_height(), -50, 10)
        volume_slider:set_on_change(function(self, value)
            print("Track " .. i .. " volume changed to: " .. value)
        end)

        table.insert(self.elements, volume_slider)
    end

    local right_rect = rect
    local top_rect = right_rect:remove_from_top(self.height * 0.2)
    
    local grid = top_rect:contracted(5):divide_grid(2, 3);
    local track_button = Button:new(grid[1][1]:get_left(), grid[1][1]:get_top(), grid[1][1]:get_width(), grid[1][1]:get_height())
    track_button:set_label("Track")
    track_button:set_on_click(function(self, x, y)
        print("Track button clicked")
    end)
    table.insert(self.elements, track_button)
    local send_button = Button:new(grid[2][1]:get_left(), grid[2][1]:get_top(), grid[2][1]:get_width(), grid[2][1]:get_height())
    send_button:set_label("Send")
    send_button:set_on_click(function(self, x, y)
        print("Send button clicked")
    end)

    table.insert(self.elements, send_button)

    local pan_button = Button:new(grid[1][2]:get_left(), grid[1][2]:get_top(), grid[1][2]:get_width(), grid[1][2]:get_height())
    pan_button:set_label("Pan")

    pan_button:set_on_click(function(self, x, y)
        print("Pan button clicked")
    end)

    table.insert(self.elements, pan_button)

    local plugin_button = Button:new(grid[2][2]:get_left(), grid[2][2]:get_top(), grid[2][2]:get_width(), grid[2][2]:get_height())
    plugin_button:set_label("Plugin")
    plugin_button:set_on_click(function(self, x, y)
        print("Plugin button clicked")
    end)

    table.insert(self.elements, plugin_button)

    local eq_button = Button:new(grid[1][3]:get_left(), grid[1][3]:get_top(), grid[1][3]:get_width(), grid[1][3]:get_height())
    eq_button:set_label("EQ")
    eq_button:set_on_click(function(self, x, y)
        print("EQ button clicked")
    end)

    table.insert(self.elements, eq_button)

    local fx_button = Button:new(grid[2][3]:get_left(), grid[2][3]:get_top(), grid[2][3]:get_width(), grid[2][3]:get_height())
    fx_button:set_label("FX")
    fx_button:set_on_click(function(self, x, y)
        print("FX button clicked")
    end)

    table.insert(self.elements, fx_button)
    
    local bottom_rect = right_rect

    local master_fader = bottom_rect:remove_from_left(50):contracted(5)
    local master_slider = Slider:new(master_fader:get_left(), master_fader:get_top(), master_fader:get_width(), master_fader:get_height(), 0, 1)
    master_slider:set_on_change(function(self, value)
        print("Master volume changed to: " .. value)
    end)

    table.insert(self.elements, master_slider)



end

local page = DefaultPage:new()

page["_AUTHOR"] = "Oscar Basic 8-Channel Controller"
page["_URL"] = "https://example.com"

return page