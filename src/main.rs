use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() {
    let pool = MySqlPoolOptions::new().max_connections(5);
    println!("Hello, world!");
}
