use dotenv::dotenv;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::env;
use tokio::sync::OnceCell;
use urlencoding::encode;

pub static POOL: OnceCell<MySqlPool> = OnceCell::const_new();

pub async fn pool() -> &'static MySqlPool {
    POOL.get_or_init(connect_to_database).await
}

pub async fn connect_to_database() -> MySqlPool {
    connect_user_sunminimart()
        .await
        .unwrap_or(init_connection().await.expect("database connection failed"))
}

pub async fn init_connection() -> sqlx::Result<MySqlPool> {
    create_sunminimart(connect_user_root().await?).await?;
    let pool = connect_user_sunminimart().await?;
    sqlx::query!(
        "
        CREATE TABLE IF NOT EXISTS products (
            ID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            Barcode VARCHAR(64) UNIQUE NOT NULL,
            Name VARCHAR(64) NOT NULL,
            Cost DECIMAL(5, 2) NOT NULL,
            Retail DECIMAL(5, 2) NOT NULL,
            Wholesale DECIMAL(5, 2) NOT NULL
        );
        ",
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
        "
        CREATE TABLE IF NOT EXISTS stocks (
            ID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            Barcode VARCHAR(64) NOT NULL,
            Expire_Date DATE NOT NULL,
            Amount SMALLINT UNSIGNED DEFAULT 0,
            FOREIGN KEY (Barcode) REFERENCES products(Barcode) ON DELETE CASCADE ON UPDATE CASCADE,
            UNIQUE (Barcode, Expire_Date)
        );
        ",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
pub async fn connect_user_root() -> sqlx::Result<MySqlPool> {
    dotenv().ok();
    let db_root_url = env::var("DB_ROOT_URL").unwrap_or_default();
    MySqlPoolOptions::new().connect(&db_root_url).await
}

pub async fn connect_user_sunminimart() -> sqlx::Result<MySqlPool> {
    dotenv().ok();
    let db_sunminimart_url = env::var("DATABASE_URL").unwrap_or_default();
    MySqlPoolOptions::new().connect(&db_sunminimart_url).await
}

pub async fn create_sunminimart(pool: MySqlPool) -> sqlx::Result<()> {
    let db_sunminimart_password = env::var("DATABASE_PASSWORD").unwrap_or_default();

    sqlx::query!("CREATE DATABASE IF NOT EXISTS sunminimart")
        .execute(&pool)
        .await?;

    sqlx::query(&format!(
        "CREATE USER IF NOT EXISTS 'sunminimart'@'localhost' IDENTIFIED BY '{}';",
        db_sunminimart_password
    ))
    .execute(&pool)
    .await?;

    sqlx::query!("GRANT ALL PRIVILEGES ON sunminimart.* TO 'sunminimart'@'localhost';")
        .execute(&pool)
        .await?;

    sqlx::query!("FLUSH PRIVILEGES;").execute(&pool).await?;

    Ok(())
}
