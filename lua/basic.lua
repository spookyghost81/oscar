local class = require 'lib/middleclass'

local Element = require 'lib/element'
local Button = require 'lib/button'
local Color = require 'lib/color'
local Page = require 'lib/page'
local Slider = require 'lib/slider'
local Rect = require 'lib/rect'
local Fader = require 'fader'

local ChannelStrip = class('ChannelStrip', Page)

function ChannelStrip:initialize(i, x, y, width, height)
    Page.initialize(self, x, y, width, height)
    self.track_index = i
    local strip_rect = Rect:new(x, y, width, height)
    local rec_button_rect = strip_rect:remove_from_top(30):contracted(5)
    local rec_button = Button:new(rec_button_rect:get_left(), rec_button_rect:get_top(), rec_button_rect:get_width(), rec_button_rect:get_height())
    rec_button:set_label("REC")
    rec_button:set_on_click(function(self, x, y)
        local record_armed = 1.0
        if oscar.daw.tracks[i] and oscar.daw.tracks[i].record_armed then
            record_armed = 0.0
        end
        oscar:track_control(i, "record_arm", record_armed)
        print("Track " .. i .. " REC button clicked, daw is " .. (oscar.daw.tracks[i] and (oscar.daw.tracks[i].record_armed and "recording" or "not recording") or "nil"))
    end)
    rec_button:set_on_update(function(self, dt)
        -- print("Updating REC button for track " .. i)
        local track = oscar.daw.tracks[i]
        if track and track.record_armed then
            self:set_bg_color(Color:new(255,0,0))
        else
            self:set_bg_color(Color:new(100, 100, 100))
        end
    end)

    table.insert(self.children, rec_button)

    local solo_button_rect = strip_rect:remove_from_top(30):contracted(5)
    local solo_button = Button:new(solo_button_rect:get_left(), solo_button_rect:get_top(), solo_button_rect:get_width(), solo_button_rect:get_height())
    solo_button:set_label("S")
    solo_button:set_on_click(function(self, x, y)
        print("Track " .. i .. " SOLO button clicked")
        local solo = 1.0
        if oscar.daw.tracks[i] and oscar.daw.tracks[i].solo then
            solo = 0.0
        end
        oscar:track_control(i, "solo", solo)
    end)
    solo_button:set_on_update(function(self, dt)
        local track = oscar.daw.tracks[i]
        if track and track.solo then
            self:set_bg_color(Color:new(0,255,0))
        else
            self:set_bg_color(Color:new(100, 100, 100))
        end
    end)

    table.insert(self.children, solo_button)

    local mute_button_rect = strip_rect:remove_from_top(30):contracted(5)
    local mute_button = Button:new(mute_button_rect:get_left(), mute_button_rect:get_top(), mute_button_rect:get_width(), mute_button_rect:get_height())
    mute_button:set_label("M")
    mute_button:set_on_click(function(self, x, y)
        print("Track " .. i .. " MUTE button clicked")
        local mute = 1.0
        if oscar.daw.tracks[i] and oscar.daw.tracks[i].mute then
            mute = 0.0
        end
        oscar:track_control(i, "mute", mute)
    end)
    mute_button:set_on_update(function(self, dt)
        local track = oscar.daw.tracks[i]
        if track and track.mute then
            self:set_bg_color(Color:new(255,255,0))
        else
            self:set_bg_color(Color:new(100, 100, 100))
        end
    end)

    table.insert(self.children, mute_button)

    local sel_button_rect = strip_rect:remove_from_top(30):contracted(5)
    local sel_button = Button:new(sel_button_rect:get_left(), sel_button_rect:get_top(), sel_button_rect:get_width(), sel_button_rect:get_height())
    sel_button:set_label("SEL")
    sel_button:set_on_click(function(self, x, y)
        print("Track " .. i .. " SEL button clicked")
    end)

    table.insert(self.children, sel_button)

    local volume_slider_rect = strip_rect:contracted(5)

    local volume_fader = Fader:new(
        volume_slider_rect:get_left(), 
        volume_slider_rect:get_top(), 
        volume_slider_rect:get_width(), 
        volume_slider_rect:get_height(), 
        -50, 10)
    
    volume_fader:set_on_update(function(self, dt)
        local track = oscar.daw.tracks[i]
        --print("Updating volume fader for track " .. i)
        if track then
            self:update_vu(track.vu_lr, track.vu_l, track.vu_r)
            self:update_value(track.volume)
        end
    end)

    volume_fader:set_on_change(function(self, value)
        print("Track " .. i .. " volume changed to: " .. value)
    end)

    table.insert(self.children, volume_fader)
end

local DefaultPage = class('DefaultPage', Page)

function DefaultPage:initialize(width, height)
    Page.initialize(self,0,0, width, height)
    
    local rect = self.rect
    local left_rect = rect:remove_from_left(self.rect:get_width() * 0.6)

    local columns = left_rect:divide_horizontal(8)
    for i, column in ipairs(columns) do
        print("Creating channel strip for track " .. i)
        local channel_strip = ChannelStrip:new(i, column:get_left(), column:get_top(), column:get_width(), column:get_height())
        table.insert(self.children, channel_strip)
    end

    local right_rect = rect
    local top_rect = right_rect:remove_from_top(self.rect:get_height() * 0.2)
    
    local grid = top_rect:contracted(5):divide_grid(2, 3);
    local track_button = Button:new(grid[1][1]:get_left(), grid[1][1]:get_top(), grid[1][1]:get_width(), grid[1][1]:get_height())
    track_button:set_label("Track")
    track_button:set_on_click(function(self, x, y)
        print("Track button clicked")
    end)
    table.insert(self.children, track_button)
    local send_button = Button:new(grid[2][1]:get_left(), grid[2][1]:get_top(), grid[2][1]:get_width(), grid[2][1]:get_height())
    send_button:set_label("Send")
    send_button:set_on_click(function(self, x, y)
        print("Send button clicked")
    end)

    table.insert(self.children, send_button)

    local pan_button = Button:new(grid[1][2]:get_left(), grid[1][2]:get_top(), grid[1][2]:get_width(), grid[1][2]:get_height())
    pan_button:set_label("Pan")

    pan_button:set_on_click(function(self, x, y)
        print("Pan button clicked")
    end)

    table.insert(self.children, pan_button)

    local plugin_button = Button:new(grid[2][2]:get_left(), grid[2][2]:get_top(), grid[2][2]:get_width(), grid[2][2]:get_height())
    plugin_button:set_label("Plugin")
    plugin_button:set_on_click(function(self, x, y)
        print("Plugin button clicked")
    end)

    table.insert(self.children, plugin_button)

    local eq_button = Button:new(grid[1][3]:get_left(), grid[1][3]:get_top(), grid[1][3]:get_width(), grid[1][3]:get_height())
    eq_button:set_label("EQ")
    eq_button:set_on_click(function(self, x, y)
        print("EQ button clicked")
        print("GLOBAL oscar.daw.tracks[1].volume: " .. oscar.daw.tracks[1].volume)
    end)

    table.insert(self.children, eq_button)

    local fx_button = Button:new(grid[2][3]:get_left(), grid[2][3]:get_top(), grid[2][3]:get_width(), grid[2][3]:get_height())
    fx_button:set_label("FX")
    fx_button:set_on_click(function(self, x, y)
        print("FX button clicked")
    end)

    table.insert(self.children, fx_button)
    
    local bottom_rect = right_rect

    local master_fader = bottom_rect:remove_from_left(50):contracted(5)
    local master_slider = Slider:new(master_fader:get_left(), master_fader:get_top(), master_fader:get_width(), master_fader:get_height(), 0, 1)
    master_slider:set_on_change(function(self, value)
        print("Master volume changed to: " .. value)
    end)

    table.insert(self.children, master_slider)
end

function DefaultPage:update(dt)
    Page.update(self, dt)
    -- Here you can add any additional update logic for the page if needed
end

function DefaultPage:draw(ctx) 
    ctx:draw_filled_rect(0, 0, self.rect:get_width(), self.rect:get_height(), Color.BLACK)
    Page.draw(self, ctx)
    ctx:set_text_align("left", "bottom");
    ctx:draw_text("Mouse State: " .. (self.previous_mouse_state and ("x: " .. self.previous_mouse_state.x .. ", y: " .. self.previous_mouse_state.y .. ", left_pressed: " .. tostring(self.previous_mouse_state.left_pressed) .. ", right_pressed: " .. tostring(self.previous_mouse_state.right_pressed)) or "nil"), 10, self.rect:get_height() - 30)
end

print("DefaultPage initialized with size: " .. oscar.window_size[1] .. "x" .. oscar.window_size[2])
local page = DefaultPage:new(oscar.window_size[1], oscar.window_size[2])

page["_AUTHOR"] = "Oscar Basic 8-Channel Controller"
page["_URL"] = "https://example.com"

return page