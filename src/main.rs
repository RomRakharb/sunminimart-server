use axum::{routing::get, Router};

use sunminimart_server::api::get_price;

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let prob = get_price("0").await?;
    println!("{:?}", prob);

    // let app = Router::new()
    //     .route("/product", get(|| async { "Hello, World!" }))
    //     .route("/exp", get(|| async { "Hello, World!" }));

    // let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // axum::serve(listener, app).await.unwrap();

    Ok(())
}
