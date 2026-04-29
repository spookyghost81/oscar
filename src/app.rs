use crate::ui_script::{
    DrawCommand, HorizontalAlign, LuaGraphics, LuaRenderer, UiScript, VerticalAlign,
};
use anyhow::Result;
use macroquad::prelude::*;

pub struct ApplicationState {
    ui_script: UiScript,

    current_font: String,
    current_font_size: f32,
    current_text_color: Color,
    current_text_align: (HorizontalAlign, VerticalAlign),
    fonts: std::collections::HashMap<String, Option<Font>>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        let ui_script = UiScript::new("basic").unwrap();
        Self {
            ui_script,
            current_font: "Arial".to_string(),
            current_font_size: 12.0,
            current_text_color: WHITE,
            current_text_align: (HorizontalAlign::Left, VerticalAlign::Top),
            fonts: std::collections::HashMap::new(),
        }
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
            self.load_fonts().await.unwrap();
            next_frame().await
        }
    }

    async fn load_fonts(&mut self) -> Result<()> {
        for (key, font) in self.fonts.iter_mut() {
            if font.is_none() {
                let font_path = format!("./fonts/{}.ttf", key);
                let loaded_font = load_ttf_font(&font_path).await?;
                *font = Some(loaded_font);
            }
        }
        Ok(())
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
                DrawCommand::SetFont { font_name, size } => {
                    self.current_font = font_name.clone();
                    self.current_font_size = *size;
                }
                DrawCommand::SetTextColor { color } => {
                    self.current_text_color = (*color).into();
                }
                DrawCommand::SetTextAlign { h_align, v_align } => {
                    self.current_text_align = (h_align.clone(), v_align.clone());
                }

                DrawCommand::DrawCircle {
                    x,
                    y,
                    radius,
                    color,
                } => {
                    draw_circle(*x, *y, *radius, (*color).into());
                }
                DrawCommand::DrawText { text, x, y } => {
                    // let font = self.fonts.get(&self.current_font);
                    // if font.is_none() {
                    //     self.fonts.insert(self.current_font.clone(), None);
                    //     continue;
                    // }
                    // let font = font.unwrap();
                    // if font.is_none() {
                    //     continue;
                    // }
                    // let font = font.as_ref().unwrap();

                    let params = TextParams {
                        font: None,
                        font_size: self.current_font_size as u16,
                        color: self.current_text_color.into(),
                        ..Default::default()
                    };
                    let h_align = self.current_text_align.0.clone();
                    let v_align = self.current_text_align.1.clone();
                    let text_dimensions =
                        measure_text(text, None, self.current_font_size as u16, 1.0);
                    let mut draw_x = *x;
                    let mut draw_y = *y;

                    match h_align {
                        HorizontalAlign::Left => {}
                        HorizontalAlign::Center => {
                            draw_x -= text_dimensions.width / 2.0;
                        }
                        HorizontalAlign::Right => {
                            draw_x -= text_dimensions.width;
                        }
                    }

                    match v_align {
                        VerticalAlign::Top => {}
                        VerticalAlign::Center => {
                            draw_y += text_dimensions.height / 2.0;
                        }
                        VerticalAlign::Bottom => {
                            draw_y += text_dimensions.height;
                        }
                    }

                    draw_text_ex(text, draw_x, draw_y, params);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
