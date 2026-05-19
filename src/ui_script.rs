use anyhow::Context;
use anyhow::Result;
use log::debug;
use mlua::Error as LuaError;
use mlua::LuaSerdeExt;
use mlua::ObjectLike;

use crate::daw_control::DawControlMessage;
use crate::daw_control::DawState;
use crate::daw_control::TrackControl;

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
                debug!("draw_outlined_rect called with x={}, y={}, width={}, height={}, color=({}, {}, {}, {})", 
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
                debug!("draw_filled_rect called with x={}, y={}, width={}, height={}, color=({}, {}, {}, {})", x, y, width, height, color.r, color.g, color.b, color.a);
                this.draw_filled_rect(x, y, width, height, color);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_font",
            |_, this, (font_name, size): (String, f32)| {
                debug!("set_font called with font_name={}, size={}", font_name, size);
                this.set_font(&font_name, size);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_text_color",
            |_, this, color: Color| {
                debug!("set_text_color called with color=({}, {}, {}, {})", color.r, color.g, color.b, color.a);
                this.set_text_color(color);
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_text_align",
            |_, this, (h_align, v_align): (String, String)| {
                debug!("set_text_align called with h_align={}, v_align={}", h_align, v_align);
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
                debug!("draw_text called with text='{}', x={}, y={}", text, x, y);
                this.draw_text(&text, x, y);
                Ok(())
            },
        );
        methods.add_method_mut(
            "draw_circle",
            |_, this, (x, y, radius, color): (f32, f32, f32, Color)| {
                debug!(
                    "draw_circle called with x={}, y={}, radius={}, color=({}, {}, {}, {})",
                    x,
                    y,
                    radius,
                    color.r,
                    color.g,
                    color.b,
                    color.a
                );
                this.draw_circle(x, y, radius, color);
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
    pub global_state: GlobalScriptState,
    pub loaded_page: mlua::Table,
}

#[derive(Debug, Clone)]
pub struct GlobalScriptState {
    pub window_size: (f32, f32),
    pub daw: DawState,
    pub tx: std::sync::mpsc::Sender<DawControlMessage>,
}

impl mlua::UserData for GlobalScriptState {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("window_size", |lua, this| lua.to_value(&this.window_size).map_err(|e| LuaError::external(e)));
        fields.add_field_method_get("window_width", |_, this| Ok(this.window_size.0));
        fields.add_field_method_get("window_height", |_, this| Ok(this.window_size.1));
        fields.add_field_method_get("daw", |lua, this| Ok(
            lua.to_value(&this.daw).unwrap()
        ));
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("track_control", |_lua, this: &GlobalScriptState, (track_index, control_type, value): (usize, String, mlua::Value)| {
            if track_index == 0 || track_index > this.daw.tracks.len() {
                return Err(LuaError::FromLuaConversionError {
                    from: "number",
                    to: "valid track index".to_string(),
                    message: Some(format!("track index {} is out of bounds", track_index)),
                })
            }

            let value:f32 = match value {
                mlua::Value::Boolean(b) => if b { 1.0 } else { 0.0 },
                mlua::Value::Integer(i) => i as f32,
                mlua::Value::Number(n) => n as f32,
                other => {
                    return Err(LuaError::FromLuaConversionError {
                        from: other.type_name(),
                        to: "number or boolean".to_string(),
                        message: Some("expected a number or boolean value".to_string()),
                    })
                }
            };

            let control = DawControlMessage::track_control(track_index, &control_type, value)?;
            this.tx.send(control).unwrap();
            log::debug!("Received track message for track {} {} {:?}", track_index, control_type, value);
            Ok(())
        });

        methods.add_method("playback_control", |lua, this: &GlobalScriptState, (control_type, value): (String, mlua::Value)| {
            log::debug!("Received playback message {} {:?}", control_type, value);
            Ok(())
        });
    }
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

impl UiScript {
    pub fn new(filename: &str, tx: std::sync::mpsc::Sender<DawControlMessage>) -> Result<Self> {
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
        let global_state = GlobalScriptState {
            window_size: (1024.0, 768.0),
            daw: DawState::default(),
            tx
        };
        Self::push_global_state_internal(&lua, &global_state).context("Failed to update global state")?;
        let loaded_page = lua
            .load(std::path::Path::new(&format!("lua/{}.lua", filename)))
            .eval::<mlua::Table>().context("Failed to load Lua UI script")?;
        Ok(Self {
            filename: filename.to_string(),
            lua,
            global_state,
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

    pub fn update(&mut self, dt: f32) -> Result<()> {
        self.lua.scope(|scope| {
            self.loaded_page.call_method::<()>("update", dt)?;
            Ok(())
        })?;
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

    pub fn global_state(&self) -> &GlobalScriptState {
        &self.global_state
    }

    pub fn global_state_mut(&mut self) -> &mut GlobalScriptState {
        &mut self.global_state
    }

    pub fn push_global_state(&mut self) -> Result<()> {
        Self::push_global_state_internal(&self.lua, &self.global_state)
    }

    fn push_global_state_internal(lua: &mlua::Lua, global_state: &GlobalScriptState) -> Result<()> {
        let state = lua.create_userdata(global_state.clone())?;
        lua.globals().set("oscar", state)?;
        Ok(())
    }

    pub fn update_input(
        &mut self, 
        mouse_x: f32, 
        mouse_y: f32, 
        left_pressed: bool, 
        right_pressed: bool, 
        left_down: bool, 
        right_down: bool) -> Result<()> {
        let mouse_state = MouseState {
            x: mouse_x,
            y: mouse_y,
            left_down,
            left_pressed,
            right_down,
            right_pressed,
        };
        
        let mouse_state = self.lua.to_value(&mouse_state)?;
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
