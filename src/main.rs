mod app;
mod graphics;
mod config;
mod world;
mod utils;
mod camera;

use log::{debug};
use crate::app::App;
use crate::config::config::load_config;

#[tokio::main()]
async fn main() {
    pretty_env_logger::init();

    let config = load_config().await;
    debug!("{:?}", config);

    let mut app = App::new(config);
    app.run().unwrap();
}
