mod connection;
mod handler;
mod pitfalls;
mod state;
mod tests;

use axum::routing::{delete, get, post, put, Router};
use dotenv::dotenv;
use std::error::Error;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv()?;

    // initialize tracing -> INFO / DEBUG modes
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = connection::conn().await?;

    // build our application with a route
    let app = Router::new()
        .route("/spells", post(handler::create))
        .route("/spells", get(handler::list))
        .route("/spells/:id", get(handler::read))
        .route("/spells/:id", put(handler::update))
        .route("/spells/:id", delete(handler::delete))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
