use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use sunminimart_server::{api, database};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = database::connect_to_database().await;
    let pool = Arc::new(Mutex::new(pool));

    let app = Router::new()
        .route("/product", post(api::add_product))
        .route("/product/{barcode}", delete(api::delete_product))
        .route("/price/{barcode}", get(api::get_price))
        .route("/stock/{barcode}", get(api::restock))
        .route("/sell", post(api::sell))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
