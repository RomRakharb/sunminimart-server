use std::sync::Arc;

use axum::{
    body::Body,
    extract::{self, Path, State},
    http::Response,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;
use sqlx::{types::chrono::NaiveDate, MySql, MySqlPool, Transaction};
use sqlx::{types::BigDecimal, Pool};
use tokio::sync::{Mutex, MutexGuard};

use crate::database::pool;

async fn begin_transaction<'a>(
    pool: &'a MutexGuard<'a, Pool<MySql>>,
) -> Result<Transaction<'a, MySql>, Response<Body>> {
    match pool.begin().await {
        Ok(transaction) => Ok(transaction),
        Err(e) => {
            eprintln!("Failed to begin transaction: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to begin transaction: {}", e),
            )
                .into_response())
        }
    }
}

async fn commit_transaction(transaction: Transaction<'_, MySql>) -> Result<(), Response<Body>> {
    match transaction.commit().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to commit transaction: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to commit transaction: {}", e),
            )
                .into_response())
        }
    }
}

async fn rollback_transaction(transaction: Transaction<'_, MySql>) -> Result<(), Response<Body>> {
    match transaction.rollback().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to rollback transaction: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to rollback transaction: {}", e),
            )
                .into_response())
        }
    }
}

#[derive(Deserialize)]
pub struct AddProduct {
    barcode: String,
    name: String,
    cost: Decimal,
    price: u16,
    amount: u16,
    expire_dates: Vec<[u32; 3]>, // NaiveDate::from_ymd_opt(year, month, day)
}
pub async fn add_product(
    State(pool): State<Arc<Mutex<MySqlPool>>>,
    extract::Json(payload): extract::Json<AddProduct>,
) -> impl IntoResponse {
    let pool = pool.lock().await;
    let mut transaction = match begin_transaction(&pool).await {
        Ok(transaction) => transaction,
        Err(response) => return response,
    };

    let add_product = sqlx::query!(
        "
            INSERT INTO products
                (Barcode, Name, Price)
            VALUES (?, ?, ?);
            ",
        payload.barcode,
        payload.name,
        payload.price
    )
    .execute(&mut *transaction)
    .await;

    let add_stocks = sqlx::query!(
        "
            INSERT INTO stocks
                (Barcode, Cost, amount)
            VALUES (?, ?, ?);
            ",
        payload.barcode,
        payload.cost,
        payload.amount
    )
    .execute(&mut *transaction)
    .await;

    let mut add_exp = Vec::new();
    for exp in payload.expire_dates {
        if let Some(date) = NaiveDate::from_ymd_opt(exp[0] as i32, exp[1], exp[2]) {
            add_exp.push(
                sqlx::query!(
                    "
            INSERT INTO expire_dates
                (Barcode, ExpireDate)
            VALUES (?, ?);
            ",
                    payload.barcode,
                    date
                )
                .execute(&mut *transaction)
                .await,
            );
        }
    }

    if [add_product, add_stocks]
        .into_iter()
        .chain(add_exp)
        .all(|res| res.is_ok())
    {
        if let Err(e) = commit_transaction(transaction).await {
            return e;
        }
        (axum::http::StatusCode::OK, "Product added successfully").into_response()
    } else {
        if let Err(e) = rollback_transaction(transaction).await {
            return e;
        }
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Error adding product",
        )
            .into_response()
    }
}

pub async fn get_price(Path(barcode): Path<String>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Product {
        pub name: String,
        pub price: u16,
    }

    match sqlx::query_as!(
        Product,
        "
        SELECT
            Name AS name,
            Price AS price
        FROM products
        WHERE Barcode = ?
        ",
        barcode
    )
    .fetch_optional(pool().await)
    .await
    {
        Ok(product) => (axum::http::StatusCode::OK, Json(product)).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting price",
        )
            .into_response(),
    }
}

pub async fn restock(
    barcode: &str,
    cost: BigDecimal,
    amount: u16,
    expire_dates: Vec<NaiveDate>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO stocks
            (Barcode, Cost, Amount)
        VALUES (?, ?, ?)
        ",
        barcode,
        cost,
        amount,
    )
    .execute(pool().await)
    .await?;

    for expire_date in expire_dates {
        sqlx::query!(
            "
        INSERT INTO expire_dates (Barcode, ExpireDate)
        VALUES (?, ?)
        ",
            barcode,
            expire_date
        )
        .execute(pool().await)
        .await?;
    }

    Ok(())
}

pub async fn delete_product(barcode: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "
        DELETE FROM products
        WHERE Barcode = ?
        ",
        barcode
    )
    .execute(pool().await)
    .await?;

    Ok(())
}

// pub async fn sell(barcode: &str) -> sqlx::Result<()> {
//     let amount = sqlx::query_as!(
//         "
//         SELECT
//             Name AS name,
//             Price AS price
//         FROM products
//         WHERE Barcode = ?
//         "
//     );

//     sqlx::query!(
//         "
//         UPDATE stocks
//         SET Amount = Amount - 1
//         WHERE Barcode = ?
//         LIMIT 1;
//         ",
//         barcode
//     )
//     .execute(pool().await)
//     .await?;
//     Ok(())
// }
