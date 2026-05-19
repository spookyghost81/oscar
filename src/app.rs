use std::sync::Arc;

use crate::daw_control::{self, DawControlMessage, DawState};
use crate::ui_script::{Color, DrawCommand, HorizontalAlign, LuaGraphics, UiScript, VerticalAlign};
use anyhow::{Context, Result};
use egui_wgpu::{Renderer as EpaintRenderer, ScreenDescriptor};
use epaint::emath::Align2;
use epaint::text::{FontDefinitions, FontFamily, FontId, Fonts, TextOptions};
use epaint::{
    ClippedShape, Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind, TessellationOptions,
    Tessellator, TextureId, pos2, vec2,
};
use rosc::OscMessage;
use winit::dpi::PhysicalSize;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct ApplicationState {
    pub window_size: (f32, f32),
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub painter: EpaintRenderer,
    pub fonts: Fonts,
    pub text_options: TextOptions,
    pub pixels_per_point: f32,

    pub mouse_position: (f32, f32),
    pub mouse_buttons: [bool; 5],
    pub mouse_buttons_pressed: [bool; 5],

    receiver: std::sync::mpsc::Receiver<OscMessage>,
    sender: std::sync::mpsc::Sender<DawControlMessage>,

    ui_script: Result<UiScript>,
}

impl ApplicationState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
        receiver: std::sync::mpsc::Receiver<OscMessage>,
        sender: std::sync::mpsc::Sender<DawControlMessage>,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find a suitable GPU adapter")
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                ..Default::default()
            })
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let preferred_format = wgpu::TextureFormat::Bgra8Unorm;
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| *format == preferred_format)
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let text_options = TextOptions::default();
        let fonts = Fonts::new(text_options.clone(), FontDefinitions::default());
        let painter = EpaintRenderer::new(&device, surface_config.format, Default::default());
        let sender2 = sender.clone();
        Self {
            window_size: (width as f32, height as f32),
            device,
            queue,
            surface_config,
            surface,
            painter,
            fonts,
            text_options,
            pixels_per_point: window.scale_factor() as f32,
            mouse_position: (0.0, 0.0),
            mouse_buttons: [false; 5],
            mouse_buttons_pressed: [false; 5],
            receiver,
            sender,
            ui_script: UiScript::new("basic", sender2),
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.window_size = (width as f32, height as f32);
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn current_clip_rect(&self) -> Rect {
        Rect::from_min_size(
            pos2(0.0, 0.0),
            vec2(
                self.surface_config.width as f32 / self.pixels_per_point,
                self.surface_config.height as f32 / self.pixels_per_point,
            ),
        )
    }

    fn build_shapes(&mut self, commands: &[DrawCommand]) -> Vec<ClippedShape> {
        self.fonts.begin_pass(self.text_options.clone());

        let clip_rect = self.current_clip_rect();
        let mut fonts_view = self.fonts.with_pixels_per_point(self.pixels_per_point);
        let mut current_font_size = 16.0;
        let mut current_text_color = Color32::WHITE;
        let mut current_text_anchor = Align2::LEFT_TOP;
        let mut shapes = Vec::with_capacity(commands.len());

        for command in commands {
            let shape = match command {
                DrawCommand::DrawFilledRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => Some(Shape::rect_filled(
                    Rect::from_min_size(pos2(*x, *y), vec2(*width, *height)),
                    CornerRadius::ZERO,
                    to_color32(*color),
                )),
                DrawCommand::DrawOutlinedRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => Some(Shape::rect_stroke(
                    Rect::from_min_size(pos2(*x, *y), vec2(*width, *height)),
                    CornerRadius::ZERO,
                    Stroke::new(2.0, to_color32(*color)),
                    StrokeKind::Inside,
                )),
                DrawCommand::DrawCircle {
                    x,
                    y,
                    radius,
                    color,
                } => Some(Shape::circle_filled(
                    pos2(*x, *y),
                    *radius,
                    to_color32(*color),
                )),
                DrawCommand::SetFont { font_name: _, size } => {
                    current_font_size = *size;
                    None
                }
                DrawCommand::SetTextColor { color } => {
                    current_text_color = to_color32(*color);
                    None
                }
                DrawCommand::SetTextAlign { h_align, v_align } => {
                    current_text_anchor = to_align2(h_align, v_align);
                    None
                }
                DrawCommand::DrawText { text, x, y } => Some(Shape::text(
                    &mut fonts_view,
                    pos2(*x, *y),
                    current_text_anchor,
                    text,
                    FontId::new(current_font_size, FontFamily::Proportional),
                    current_text_color,
                )),
            };

            if let Some(shape) = shape {
                shapes.push(ClippedShape { clip_rect, shape });
            }
        }

        shapes
    }

    fn draw_commands(&mut self, commands: &[DrawCommand], target_view: &wgpu::TextureView) {
        let clipped_shapes = self.build_shapes(commands);

        let mut tessellator = Tessellator::new(
            self.pixels_per_point,
            TessellationOptions::default(),
            self.fonts.font_image_size(),
            Vec::new(),
        );
        let paint_jobs = tessellator.tessellate_shapes(clipped_shapes);

        if let Some(font_delta) = self.fonts.font_image_delta() {
            self.painter.update_texture(
                &self.device,
                &self.queue,
                TextureId::default(),
                &font_delta,
            );
        }

        let screen = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.pixels_per_point,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("epaint encoder"),
            });

        let extra_command_buffers = self.painter.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("epaint render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            let mut render_pass = render_pass.forget_lifetime();
            self.painter.render(&mut render_pass, &paint_jobs, &screen);
        }

        self.queue.submit(
            extra_command_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
    }

    fn render_script(&mut self) {
        let mut graphics = LuaGraphics::new();

        if let Ok(ui_script) = self.ui_script.as_mut() {
            if let Err(error) = ui_script
                .update_input(
                    self.mouse_position.0,
                    self.mouse_position.1,
                    self.mouse_buttons_pressed[0],
                    self.mouse_buttons_pressed[1],
                    self.mouse_buttons[0],
                    self.mouse_buttons[1],
                )
                .and_then(|_| {
                    self.mouse_buttons_pressed = [false; 5];
                    ui_script.update(1.0 / 60.0)?;
                    ui_script.draw(&mut graphics)
                })
            {
                log::error!("Error running UI script: {error:?}");
            }
        } else if let Err(error) = &self.ui_script {
            log::error!("UI script is unavailable: {error:?}");
        }

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                log::warn!("Timed out while acquiring the next surface texture");
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::warn!("Surface validation failed while acquiring the next frame");
                return;
            }
        };

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.draw_commands(&graphics.commands, &surface_view);
        surface_texture.present();
    }
}

pub struct App {
    instance: wgpu::Instance,
    state: Option<ApplicationState>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
            window: None,
        }
    }

    pub fn handle_messages(&mut self) {
        if let Some(state) = self.state.as_mut() {
            while let Ok(msg) = state.receiver.try_recv() {
                let _ = state.ui_script.as_mut().map(|ui_script| {
                    ui_script
                        .global_state_mut()
                        .daw
                        .update_from_osc_message(msg);
                });
            }
            let res = state.ui_script.as_mut().map(|ui_script| {
                ui_script.push_global_state().unwrap();
            });
            if res.is_err() {
                log::error!("Failed to push global state to UI script: {:?}", res.err());
            }
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1024;
        let initial_height = 768;
        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Unable to create surface");
        let (incoming_packet_sender, incoming_packet_receiver) = std::sync::mpsc::channel();
        let (outgoing_packet_sender, outgoing_packet_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            match daw_control::network_thread(incoming_packet_sender, outgoing_packet_receiver) {
                Ok(()) => log::info!("Network thread exited successfully"),
                Err(e) => {
                    log::error!("Network thread exited with error: {:?}", e);
                    panic!("Network thread exited with error: {:?}", e);
                }
            }
        });
        let state = ApplicationState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_height,
            incoming_packet_receiver,
            outgoing_packet_sender,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_key(&mut self, key_code: KeyCode, is_pressed: bool) {
        if let (KeyCode::Escape, true) = (key_code, is_pressed) {
            log::info!("Escape key pressed. Exiting application.");
            std::process::exit(0);
        }
        if let (KeyCode::KeyR, true) = (key_code, is_pressed) {
            log::info!("R key pressed. Reloading UI script.");
            if let Some(state) = self.state.as_mut() {
                state.ui_script = UiScript::new("basic", state.sender.clone());
            }
        }
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(state) = self.state.as_mut() {
                state.resize_surface(width, height);
                state.window_size = (width as f32, height as f32);
            }
        }
    }

    fn handle_redraw(&mut self) {
        if let Some(window) = self.window.as_ref() {
            if let Some(true) = window.is_minimized() {
                log::info!("Window is minimized");
                return;
            }
        }

        if let Some(state) = self.state.as_mut() {
            let _ = state
                .ui_script
                .as_mut()
                .map(|ui_script| ui_script.global_state_mut().window_size = state.window_size);
        }

        self.handle_messages();
        let state = self
            .state
            .as_mut()
            .expect("Application state should be initialized");

        if let Some(window) = self.window.as_ref() {
            state.pixels_per_point = window.scale_factor() as f32;
        }
        state.render_script();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        pollster::block_on(self.set_window(window));

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.state.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => self.handle_key(KeyCode::Escape, true),
            WindowEvent::Resized(size) => self.handle_resized(size.width, size.height),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(state) = self.state.as_mut() {
                    state.pixels_per_point = scale_factor as f32;
                }
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if let Some(appstate) = self.state.as_mut() {
                    let button_index = match button {
                        winit::event::MouseButton::Left => 0,
                        winit::event::MouseButton::Right => 1,
                        winit::event::MouseButton::Middle => 2,
                        winit::event::MouseButton::Other(index) if index < 5 => index as usize,
                        _ => return,
                    };
                    appstate.mouse_buttons[button_index] = state.is_pressed();
                    if state.is_pressed() {
                        appstate.mouse_buttons_pressed[button_index] = true;
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some(state) = self.state.as_mut() {
                    state.mouse_position = (position.x as f32, position.y as f32);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(code, key_state.is_pressed()),
            _ => {}
        }
    }
}

fn to_color32(color: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn to_align2(h_align: &HorizontalAlign, v_align: &VerticalAlign) -> Align2 {
    match (h_align, v_align) {
        (HorizontalAlign::Left, VerticalAlign::Top) => Align2::LEFT_TOP,
        (HorizontalAlign::Center, VerticalAlign::Top) => Align2::CENTER_TOP,
        (HorizontalAlign::Right, VerticalAlign::Top) => Align2::RIGHT_TOP,
        (HorizontalAlign::Left, VerticalAlign::Center) => Align2::LEFT_CENTER,
        (HorizontalAlign::Center, VerticalAlign::Center) => Align2::CENTER_CENTER,
        (HorizontalAlign::Right, VerticalAlign::Center) => Align2::RIGHT_CENTER,
        (HorizontalAlign::Left, VerticalAlign::Bottom) => Align2::LEFT_BOTTOM,
        (HorizontalAlign::Center, VerticalAlign::Bottom) => Align2::CENTER_BOTTOM,
        (HorizontalAlign::Right, VerticalAlign::Bottom) => Align2::RIGHT_BOTTOM,
    }
}

// impl ApplicationState {
//     pub fn draw_error(error_message: &str) {
//         let params = TextParams {
//             font: None,
//             font_size: 20,
//             color: RED,
//             ..Default::default()
//         };
//         draw_text_ex(&error_message, 10.0, 30.0, params);
//     }

//     pub fn run_script(&mut self) -> Result<()> {
//         let mut graphics = LuaGraphics::new();
//         let ui_script = self
//             .ui_script
//             .as_mut()
//             .map_err(|e| anyhow::anyhow!("Failed to load UI script: {:?}", e))?;
//         ui_script.handle_input()?;
//         ui_script.draw(&mut graphics)?;
//         self.render(&graphics.commands)?;

//         Ok(())
//     }

//     pub async fn run(&mut self) {
//         loop {
//             if is_key_pressed(KeyCode::R) {
//                 if let Ok(ui_script) = &mut self.ui_script {
//                     ui_script.reload().unwrap();
//                 } else {
//                     self.ui_script = UiScript::new("basic");
//                 }
//                 self.is_running = true;
//                 self.error_message = None;
//             }

//             clear_background(BLACK);

//             if self.is_running {
//                 match self.run_script() {
//                     Ok(()) => (),
//                     Err(e) => {
//                         let error_message =
//                             format!("Error running UI script: {:?}... Press R to retry.", e);

//                         self.error_message = Some(error_message.clone());
//                         log::error!("Error running UI script: {:?}", e);
//                         self.is_running = false;

//                         next_frame().await;
//                         continue;
//                     }
//                 };
//                 self.load_fonts().await.unwrap();
//             }

//             if self.error_message.is_some() {
//                 Self::draw_error(self.error_message.as_ref().unwrap());
//             }
//             next_frame().await
//         }
//     }

//     async fn load_fonts(&mut self) -> Result<()> {
//         for (key, font) in self.fonts.iter_mut() {
//             if font.is_none() {
//                 let font_path = format!("./fonts/{}.ttf", key);
//                 let loaded_font = load_ttf_font(&font_path).await?;
//                 *font = Some(loaded_font);
//             }
//         }
//         Ok(())
//     }
// }

// impl From<crate::ui_script::Color> for macroquad::prelude::Color {
//     fn from(color: crate::ui_script::Color) -> Self {
//         macroquad::prelude::Color::new(
//             color.r as f32 / 255.0,
//             color.g as f32 / 255.0,
//             color.b as f32 / 255.0,
//             color.a as f32 / 255.0,
//         )
//     }
// }

// impl LuaRenderer for ApplicationState {
//     fn render(&mut self, commands: &[DrawCommand]) -> Result<()> {
//         for command in commands {
//             match command {
//                 DrawCommand::DrawFilledRect {
//                     x,
//                     y,
//                     width,
//                     height,
//                     color,
//                 } => {
//                     draw_rectangle(*x, *y, *width, *height, (*color).into());
//                 }
//                 DrawCommand::DrawOutlinedRect {
//                     x,
//                     y,
//                     width,
//                     height,
//                     color,
//                 } => {
//                     draw_rectangle_lines(*x, *y, *width, *height, 2.0, (*color).into());
//                 }
//                 DrawCommand::SetFont { font_name, size } => {
//                     self.current_font = font_name.clone();
//                     self.current_font_size = *size;
//                 }
//                 DrawCommand::SetTextColor { color } => {
//                     self.current_text_color = (*color).into();
//                 }
//                 DrawCommand::SetTextAlign { h_align, v_align } => {
//                     self.current_text_align = (h_align.clone(), v_align.clone());
//                 }
//                 DrawCommand::DrawCircle {
//                     x,
//                     y,
//                     radius,
//                     color,
//                 } => {
//                     draw_circle(*x, *y, *radius, (*color).into());
//                 }
//                 DrawCommand::DrawText { text, x, y } => {
//                     // let font = self.fonts.get(&self.current_font);
//                     // if font.is_none() {
//                     //     self.fonts.insert(self.current_font.clone(), None);
//                     //     continue;
//                     // }
//                     // let font = font.unwrap();
//                     // if font.is_none() {
//                     //     continue;
//                     // }
//                     // let font = font.as_ref().unwrap();

//                     let params = TextParams {
//                         font: None,
//                         font_size: self.current_font_size as u16,
//                         color: self.current_text_color.into(),
//                         ..Default::default()
//                     };
//                     let h_align = self.current_text_align.0.clone();
//                     let v_align = self.current_text_align.1.clone();
//                     let text_dimensions =
//                         measure_text(text, None, self.current_font_size as u16, 1.0);
//                     let mut draw_x = *x;
//                     let mut draw_y = *y;

//                     match h_align {
//                         HorizontalAlign::Left => {}
//                         HorizontalAlign::Center => {
//                             draw_x -= text_dimensions.width / 2.0;
//                         }
//                         HorizontalAlign::Right => {
//                             draw_x -= text_dimensions.width;
//                         }
//                     }

//                     match v_align {
//                         VerticalAlign::Top => {}
//                         VerticalAlign::Center => {
//                             draw_y += text_dimensions.height / 2.0;
//                         }
//                         VerticalAlign::Bottom => {
//                             draw_y += text_dimensions.height;
//                         }
//                     }

//                     draw_text_ex(text, draw_x, draw_y, params);
//                 }
//                 _ => {}
//             }
//         }
//         Ok(())
//     }
// }
