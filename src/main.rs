use axum::{routing::get, Router};

use sunminimart_server::api;
use sunminimart_server::database::pool;

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let _ = pool().await;

    // let _ = api::delete_product("02").await;

    // let _ = api::add_product("01", "test", 10).await;
    // let _ = api::add_product("02", "test", 20).await;
    // let _ = api::add_product("03", "test", 30).await;

    // let prob = get_price_retail("0").await?;
    // println!("{:?}", prob);

    let app = Router::new()
        .route("/product", get(|| async { "Hello, World!" }))
        .route("/exp", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
