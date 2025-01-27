// TODO: init
// Create user sunminimart **** password must be kept somewhere else!!!!!
// Connect trough user sunminimart
// Create database "sunminimart"
// Connect to the database
// Start Creating Table

use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::env;
use urlencoding::encode;

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let _pool = connect_user_sunminimart().await.unwrap_or(init().await?);

    Ok(())
}

async fn connect_user_root() -> sqlx::Result<MySqlPool> {
    let db_root_password = env::var("DB_ROOT_PASSWORD").unwrap_or_default();
    let encoded_db_root_password = encode(&db_root_password);
    let db_root_url = format!("mysql://root:{}@localhost:3306/", encoded_db_root_password);
    MySqlPoolOptions::new().connect(&db_root_url).await
}

async fn connect_user_sunminimart() -> sqlx::Result<MySqlPool> {
    let db_sunminimart_password = env::var("DB_SUNMINIMART_PASSWORD").unwrap_or_default();
    let encoded_db_sunminimart_password = encode(&db_sunminimart_password);
    let db_sunminimart_url = format!(
        "mysql://sunminimart:{}@localhost:3306/sunminimart",
        encoded_db_sunminimart_password
    );
    MySqlPoolOptions::new().connect(&db_sunminimart_url).await
}

async fn init() -> sqlx::Result<MySqlPool> {
    let pool = connect_user_root().await?;
    let db_sunminimart_password = env::var("DB_SUNMINIMART_PASSWORD").unwrap_or_default();

    sqlx::query("CREATE DATABASE IF NOT EXISTS sunminimart")
        .execute(&pool)
        .await?;
    sqlx::query(&format!(
        "CREATE USER IF NOT EXISTS 'sunminimart'@'localhost' IDENTIFIED BY '{}';",
        db_sunminimart_password
    ))
    .execute(&pool)
    .await?;
    sqlx::query("GRANT ALL PRIVILEGES ON sunminimart.* TO 'sunminimart'@'localhost';")
        .execute(&pool)
        .await?;
    sqlx::query("FLUSH PRIVILEGES;").execute(&pool).await?;

    let pool = connect_user_sunminimart().await?;
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS products (
            ID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            Barcode VARCHAR(64) UNIQUE NOT NULL,
            Name VARCHAR(64) NOT NULL,
            Cost DECIMAL(5, 2) NOT NULL,
            Retail DECIMAL(5, 2) NOT NULL,
            Wholesale DECIMAL(5, 2) NOT NULL,
            Amount SMALLINT UNSIGNED DEFAULT 0
        );
        ",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS expire_dates (
            ID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            Barcode VARCHAR(64) NOT NULL,
            Expire_Date DATE NOT NULL,
            FOREIGN KEY (Barcode) REFERENCES products(Barcode) ON DELETE CASCADE ON UPDATE CASCADE,
            UNIQUE (Barcode, Expire_Date)  -- Optional: Prevent duplicate barcodes on the same date
        );
        ",
    )
    .execute(&pool)
    .await?;

    connect_user_sunminimart().await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn db_root_password_set() {
        assert!(env::var("DB_ROOT_PASSWORD").is_ok());
    }
    #[test]
    fn db_sunminimart_password_set() {
        assert!(env::var("DB_SUNMINIMART_PASSWORD").is_ok());
    }
}
