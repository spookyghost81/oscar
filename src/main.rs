use winit::event_loop::EventLoop;

mod app;
mod daw_control;
mod ui_script;

// fn window_conf() -> macroquad::prelude::Conf {
//     macroquad::prelude::Conf {
//         window_title: "OSCar".to_string(),
//         window_width: 800,
//         window_height: 600,
//         ..Default::default()
//     }
// }

pub fn run() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = app::App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn main() {
    run().unwrap();
}
