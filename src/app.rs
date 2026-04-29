use crate::ui_script::{DrawCommand, LuaGraphics, LuaRenderer, UiScript};
use anyhow::Result;
use macroquad::prelude::*;

pub struct ApplicationState {
    ui_script: UiScript,
}

impl Default for ApplicationState {
    fn default() -> Self {
        let ui_script = UiScript::new("basic").unwrap();
        Self { ui_script }
    }
}

impl ApplicationState {
    pub async fn run(&mut self) {
        loop {
            let mut graphics = LuaGraphics::new();
            if is_key_pressed(KeyCode::R) {
                self.ui_script.reload().unwrap();
            }

            clear_background(BLACK);
            self.ui_script.draw(&mut graphics).unwrap();
            self.render(&graphics.commands).unwrap();
            next_frame().await
        }
    }
}

impl From<crate::ui_script::Color> for macroquad::prelude::Color {
    fn from(color: crate::ui_script::Color) -> Self {
        macroquad::prelude::Color::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        )
    }
}

impl LuaRenderer for ApplicationState {
    fn render(&mut self, commands: &[DrawCommand]) -> Result<()> {
        for command in commands {
            match command {
                DrawCommand::DrawFilledRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    draw_rectangle(*x, *y, *width, *height, (*color).into());
                }
                DrawCommand::DrawOutlinedRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    draw_rectangle_lines(*x, *y, *width, *height, 2.0, (*color).into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}
