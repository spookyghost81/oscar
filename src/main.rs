mod app;
mod daw_control;
mod ui_script;

fn window_conf() -> macroquad::prelude::Conf {
    macroquad::prelude::Conf {
        window_title: "OSCar".to_string(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    app::ApplicationState::default().run().await
}
