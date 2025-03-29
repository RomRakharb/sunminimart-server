use std::sync::Arc;

use axum::routing::post;
use axum::{routing::get, Router};

use sunminimart_server::{api, database};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = database::connect_to_database().await;
    let pool = Arc::new(Mutex::new(pool));

    // let _ = api::delete_product("02").await;

    // let _ = api::add_product("01", "test", 10).await;
    // let _ = api::add_product("02", "test", 20).await;
    // let _ = api::add_product("03", "test", 30).await;

    // let prob = get_price_retail("0").await?;
    // println!("{:?}", prob);

    let app = Router::new()
        .route("/product", post(api::add_product))
        .route("/price/:barcode", get(api::get_price))
        .route("/exp", get(|| async { "Hello, World!" }))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
