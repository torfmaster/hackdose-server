use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;

use crate::Configuration;

use self::visualisation::render_image;

mod visualisation;

pub(crate) async fn serve_rest_endpoint(mutex: Arc<Mutex<i32>>, config: &Configuration) {
    let owned_config = config.clone();
    let energy = warp::path("energy")
        .map(move || mutex.clone())
        .and_then(return_energy);
    let image = warp::path("day")
        .and(warp::any().map(move || owned_config.clone()))
        .and_then(image);
    warp::serve(energy.or(image))
        .run(([0, 0, 0, 0], 8080))
        .await;
}

async fn return_energy(m: Arc<Mutex<i32>>) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    Ok(Box::new(warp::reply::json(&format!(
        "Current Consumption: {}",
        m.lock().await
    ))))
}

async fn image(config: Configuration) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let svg_image = render_image(&config).await;
    Ok(Box::new(warp::reply::html(format!(
        "<html>{}</html>",
        svg_image
    ))))
}
