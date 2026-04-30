use anyhow::Result;
use log::debug;
use macroquad::prelude::*;
use mlua::Error as LuaError;
use mlua::ObjectLike;



#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl mlua::FromLua for Color {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Table(table) => Ok(Self {
                r: table.get("r")?,
                g: table.get("g")?,
                b: table.get("b")?,
                a: table.get::<Option<u8>>("a")?.unwrap_or(255),
            }),
            other => Err(LuaError::FromLuaConversionError {
                from: other.type_name(),
                to: "Color".to_string(),
                message: Some("expected a Lua Color table with r/g/b/a fields".to_string()),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    DrawOutlinedRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
    DrawFilledRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
    SetFont {
        font_name: String,
        size: f32,
    },
    SetTextColor {
        color: Color,
    },
    SetTextAlign {
        h_align: HorizontalAlign,
        v_align: VerticalAlign,
    },
    DrawText {
        text: String,
        x: f32,
        y: f32,
    },
    DrawCircle {
        x: f32,
        y: f32,
        radius: f32,
        color: Color,
    },
}

#[derive(Debug, Clone)]
pub struct LuaGraphics {
    pub commands: Vec<DrawCommand>,
}

impl LuaGraphicsContext for LuaGraphics {
    fn draw_outlined_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.commands.push(DrawCommand::DrawOutlinedRect {
            x,
            y,
            width,
            height,
            color,
        });
    }
    fn draw_filled_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.commands.push(DrawCommand::DrawFilledRect {
            x,
            y,
            width,
            height,
            color,
        });
    }
    fn set_font(&mut self, font_name: &str, size: f32) {
        self.commands.push(DrawCommand::SetFont {
            font_name: font_name.to_string(),
            size,
        });
    }
    fn set_text_color(&mut self, color: Color) {
        self.commands.push(DrawCommand::SetTextColor { color });
    }
    fn set_text_align(&mut self, h_align: HorizontalAlign, v_align: VerticalAlign) {
        self.commands
            .push(DrawCommand::SetTextAlign { h_align, v_align });
    }
    fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
    ) {
        self.commands.push(DrawCommand::DrawText {
            text: text.to_string(),
            x,
            y,
        });
    }
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.commands.push(DrawCommand::DrawCircle {
            x,
            y,
            radius,
            color,
        });
    }
}

impl LuaGraphics {
    pub fn new() -> Self {
        LuaGraphics {
            commands: Vec::new(),
        }
    }
}

impl mlua::UserData for LuaGraphics {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "draw_outlined_rect",
            |_, this, (x, y, width, height, color): (f32, f32, f32, f32, Color)| {
                debug!("drawOutlinedRect called with x={}, y={}, width={}, height={}, color=({}, {}, {}, {})", 
                x, y,
                 width, height, 
                 color.r, color.g, color.b, color.a);
                this.draw_outlined_rect(x, y, width, height, color);
                Ok(())
            },
        );
        methods.add_method_mut(
            "draw_filled_rect",
            |_, this, (x, y, width, height, color): (f32, f32, f32, f32, Color)| {
                debug!("drawFilledRect called with x={}, y={}, width={}, height={}, color=({}, {}, {}, {})", x, y, width, height, color.r, color.g, color.b, color.a);
                this.draw_filled_rect(x, y, width, height, color);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_font",
            |_, this, (font_name, size): (String, f32)| {
                debug!("setFont called with font_name={}, size={}", font_name, size);
                this.set_font(&font_name, size);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_text_color",
            |_, this, color: Color| {
                debug!("setTextColor called with color=({}, {}, {}, {})", color.r, color.g, color.b, color.a);
                this.set_text_color(color);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_text_align",
            |_, this, (h_align, v_align): (String, String)| {
                debug!("setTextAlign called with h_align={}, v_align={}", h_align, v_align);
                let h_align = match h_align.as_str() {
                    "left" => HorizontalAlign::Left,
                    "center" => HorizontalAlign::Center,
                    "right" => HorizontalAlign::Right,
                    _ => {
                        return Err(LuaError::FromLuaConversionError {
                            from: "string",
                            to: "HorizontalAlign".to_string(),
                            message: Some(
                                "expected 'left', 'center', or 'right'".to_string(),
                            ),
                        })
                    }
                };
                let v_align = match v_align.as_str() {
                    "top" => VerticalAlign::Top,
                    "center" => VerticalAlign::Center,
                    "bottom" => VerticalAlign::Bottom,
                    _ => {
                        return Err(LuaError::FromLuaConversionError {
                            from: "string",
                            to: "VerticalAlign".to_string(),
                            message: Some("expected 'top', 'center', or 'bottom'".to_string()),
                        })
                    }
                };
                this.set_text_align(h_align, v_align);
                Ok(())
            },
        );
        methods.add_method_mut(
            "draw_text",
            |_, this, (text, x, y): (String, f32, f32)| {
                debug!("drawText called with text='{}', x={}, y={}", text, x, y);
                this.draw_text(&text, x, y);
                Ok(())
            },
        );

    }
}

pub trait LuaGraphicsContext {
    fn draw_outlined_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color);
    fn draw_filled_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color);
    fn set_font(&mut self, font_name: &str, size: f32);
    fn set_text_color(&mut self, color: Color);
    fn set_text_align(&mut self, h_align: HorizontalAlign, v_align: VerticalAlign);
    fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
    );
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color);
}

pub trait LuaRenderer {
    fn render(&mut self, commands: &[DrawCommand]) -> Result<()>;
}

pub struct UiScript {
    pub filename: String,
    pub lua: mlua::Lua,
    pub loaded_page: mlua::Table,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub left_down: bool,
    pub left_pressed: bool,
    pub right_down: bool,
    pub right_pressed: bool,
}

impl mlua::UserData for MouseState {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_get("y", |_, this| Ok(this.y));
        fields.add_field_method_get("left_down", |_, this| Ok(this.left_down));
        fields.add_field_method_get("left_pressed", |_, this| Ok(this.left_pressed));
        fields.add_field_method_get("right_down", |_, this| Ok(this.right_down));
        fields.add_field_method_get("right_pressed", |_, this| Ok(this.right_pressed));
    }
}

pub struct OscarState {
    pub mouse: MouseState
}

impl UiScript {
    pub fn new(filename: &str) -> Result<Self> {
            let lua = mlua::Lua::new();

        let require = lua
            .create_function(|lua, module: String| {
            println!("Requiring module: {}", module);

            let module = module.trim_start_matches("./");
            let path = format!("lua/{module}.lua");
            lua.load(std::path::Path::new(&path)).eval::<mlua::Value>()
        })
        .unwrap();
        lua.globals().set("require", require).unwrap();
        let loaded_page = lua
            .load(std::path::Path::new(&format!("lua/{}.lua", filename)))
            .eval::<mlua::Table>()?;
        Ok(Self {
            filename: filename.to_string(),
            lua,
            loaded_page,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let loaded_page = self
            .lua
            .load(std::path::Path::new(&format!("lua/{}.lua", self.filename)))
            .eval::<mlua::Table>()?;
        self.loaded_page = loaded_page;
        Ok(())
    }

    pub fn draw(&mut self, graphics: &mut LuaGraphics) -> Result<()> {
        self.lua.scope(|scope| {
            let userdata = scope.create_userdata_ref_mut(graphics)?;
            self.loaded_page.call_method::<()>("draw", userdata)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<()> {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_state = MouseState {
            x: mouse_x,
            y: mouse_y,
            left_down: is_mouse_button_down(macroquad::prelude::MouseButton::Left),
            left_pressed: is_mouse_button_pressed(macroquad::prelude::MouseButton::Left),
            right_down: is_mouse_button_down(macroquad::prelude::MouseButton::Right),
            right_pressed: is_mouse_button_pressed(macroquad::prelude::MouseButton::Right),
        };
        
        let mouse_state = self.lua.create_userdata(mouse_state)?;
        self.loaded_page.call_method::<()>("update_mouse_state", mouse_state)?;
        
        Ok(())
    }
}

#[test]
fn test_ui_script() {
    simple_logger::SimpleLogger::new().init().unwrap();
    let lua = mlua::Lua::new();

    let require = lua
        .create_function(|lua, module: String| {
            println!("Requiring module: {}", module);

            let module = module.trim_start_matches("./");
            let path = format!("lua/{module}.lua");
            lua.load(std::path::Path::new(&path)).eval::<mlua::Value>()
        })
        .unwrap();
    lua.globals().set("require", require).unwrap();

    let page = lua
        .load(std::path::Path::new("lua/basic.lua"))
        .eval::<mlua::Table>()
        .unwrap();

    let page_class = page
        .get::<mlua::Table>("class")
        .unwrap()
        .get::<mlua::Table>("__declaredMethods")
        .unwrap();
    debug!("Page class: {:?}", page_class);

    for pair in page_class.pairs::<String, mlua::Value>() {
        let (k, v) = pair.unwrap();
        println!("Page class field: {:?} = {:?}", k, v);
    }

    for pair in page.pairs::<String, mlua::Value>() {
        let (k, v) = pair.unwrap();
        println!("Page field: {:?} = {:?}", k, v);
    }

    let elements = page.get::<mlua::Table>("elements").unwrap();
    let _ = elements.for_each(|k: String, v: mlua::Value| {
        println!("Page element: {:?} = {:?}", k, v);
        Ok(())
    });

    let mut graphics = LuaGraphics {
        commands: Vec::new(),
    };

    lua.scope(|scope| {
        let lua_graphics = scope.create_userdata_ref_mut(&mut graphics)?;
        page.call_method::<()>("draw", lua_graphics)?;
        Ok::<_, mlua::Error>(())
    })
    .unwrap();

    for command in &graphics.commands {
        debug!("Draw command: {:?}", command);
    }

}
