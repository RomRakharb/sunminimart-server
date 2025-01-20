// TODO: init
// Create user sunminimart **** password must be kept somewhere else!!!!!
// Connect trough user sunminimart
// Create database "sunminimart"
// Connect to the database
// Start Creating Table

use sqlx::{mysql::MySqlPoolOptions, query, Executor, MySqlPool};

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let _ = init();
    Ok(())
}

async fn init() -> sqlx::Result<()> {
    let pool = MySqlPoolOptions::new()
        .connect("mysql://rom:mypassword@localhost/")
        .await?;
    pool.execute(sqlx::query("CREATE DATABASE IF NOT EXISTS sunminimart;"))
        .await?;
    Ok(())
}
