use log::debug;

use crate::app::App;
use crate::config::config::load_config;

mod app;
mod graphics;
mod config;
mod world;
mod utils;
mod camera;
mod controls;

#[tokio::main()]
async fn main() {
    pretty_env_logger::init();

    let config = load_config().await.unwrap();
    debug!("{:?}", config);

    let mut app = App::new(config);
    app.run().unwrap();
}
