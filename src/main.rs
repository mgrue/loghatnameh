use askama::filters::format;
use axum::{
    routing::get,
    routing::post,
    Router
};

use tower_http::{
    services::{ServeDir}
};

use log::info;

mod handlers;
mod db;
pub mod i18n;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let state = handlers::AppState {
        db_pool: db::init().await
    };

    db::migrate(&state.db_pool).await;

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/", post(handlers::search))
        .route("/word", get(handlers::word_details))
        .route("/add-word", get(handlers::add_word_get))
        .route("/add-word", post(handlers::add_word_post))
        .nest_service("/css", ServeDir::new("css"))
        .nest_service("/about", ServeDir::new("static"))
        .with_state(state);

    let bind_iface = std::env::var("BIND_INTERFACE").unwrap_or(String::from("0.0.0.0"));
    let bind_port = std::env::var("BIND_PORT").unwrap_or(String::from("3000"));
    let bind = format!("{}:{}", bind_iface, bind_port);

    info!("Starting server at {}", bind);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}