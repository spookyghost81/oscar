local class = require 'lib/middleclass'
local Element = require 'lib/element'
local Button = require 'lib/button'
local Color = require 'lib/color'
local Page = require 'lib/page'

local page = Page:new()

local DefaultPage = class('DefaultPage', Page)

function DefaultPage:initialize(width, height)
    Page.initialize(self, width, height)
    local hello_button = Button:new(10, 10, 100, 50)
    hello_button:setLabel("Hello")
    hello_button:setOnClick(function()
        print("Hello button clicked!")
    end)
    table.insert(self.elements, hello_button)
end

local page = DefaultPage:new()

page["_AUTHOR"] = "Oscar"
page["_URL"] = "https://example.com"

return page