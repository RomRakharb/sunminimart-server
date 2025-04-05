use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use sqlx::{types::chrono::NaiveDate, MySqlPool};
use tokio::sync::Mutex;

use crate::database::transaction;

#[derive(Deserialize)]
pub struct AddProduct {
    barcode: String,
    name: String,
    cost: BigDecimal,
    price: u16,
    amount: u16,
    // NaiveDate cannot be Deserialized, use [u32; 3] instead
    expire_dates: Vec<[u32; 3]>,
}
pub async fn add_product(
    State(pool): State<Arc<Mutex<MySqlPool>>>,
    Json(payload): Json<AddProduct>,
) -> impl IntoResponse {
    let pool = pool.lock().await;
    let mut transaction = match transaction::begin(&pool).await {
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
        if let Err(response) = transaction::commit(transaction).await {
            return response;
        }
        (axum::http::StatusCode::OK, "Product added successfully").into_response()
    } else {
        if let Err(response) = transaction::rollback(transaction).await {
            return response;
        }
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Error adding product",
        )
            .into_response()
    }
}

pub async fn get_price(
    State(pool): State<Arc<Mutex<MySqlPool>>>,
    Path(barcode): Path<String>,
) -> impl IntoResponse {
    let pool = pool.lock().await;
    let mut transaction = match transaction::begin(&pool).await {
        Ok(transaction) => transaction,
        Err(response) => return response,
    };

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
    .fetch_optional(&mut *transaction)
    .await
    {
        Ok(product) => {
            if let Err(response) = transaction::commit(transaction).await {
                return response;
            }
            (axum::http::StatusCode::OK, Json(product)).into_response()
        }
        Err(_) => {
            if let Err(response) = transaction::rollback(transaction).await {
                return response;
            }
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting price",
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct Restock {
    cost: BigDecimal,
    amount: u16,
    // NaiveDate cannot be Deserialized, use [u32; 3] instead
    expire_dates: Vec<[u32; 3]>,
}
pub async fn restock(
    State(pool): State<Arc<Mutex<MySqlPool>>>,
    Path(barcode): Path<String>,
    Json(payload): Json<Restock>,
) -> impl IntoResponse {
    let pool = pool.lock().await;
    let mut transaction = match transaction::begin(&pool).await {
        Ok(transaction) => transaction,
        Err(response) => return response,
    };

    let add_stocks = sqlx::query!(
        "
        INSERT INTO stocks
            (Barcode, Cost, amount)
        VALUES (?, ?, ?);
        ",
        barcode,
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
                    barcode,
                    date
                )
                .execute(&mut *transaction)
                .await,
            );
        }
    }

    if [add_stocks]
        .into_iter()
        .chain(add_exp)
        .all(|res| res.is_ok())
    {
        if let Err(response) = transaction::commit(transaction).await {
            return response;
        }
        (axum::http::StatusCode::OK, "Restocking successfully").into_response()
    } else {
        if let Err(response) = transaction::rollback(transaction).await {
            return response;
        }
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Error restocking product",
        )
            .into_response()
    }
}

pub async fn delete_product(
    State(pool): State<Arc<Mutex<MySqlPool>>>,
    Path(barcode): Path<String>,
) -> impl IntoResponse {
    let pool = pool.lock().await;
    let mut transaction = match transaction::begin(&pool).await {
        Ok(transaction) => transaction,
        Err(response) => return response,
    };

    match sqlx::query!(
        "
        DELETE FROM products
        WHERE Barcode = ?
        ",
        barcode
    )
    .execute(&mut *transaction)
    .await
    {
        Ok(_) => {
            if let Err(response) = transaction::commit(transaction).await {
                return response;
            }
            (axum::http::StatusCode::OK, "Product deleted successfully").into_response()
        }
        Err(_) => {
            if let Err(response) = transaction::rollback(transaction).await {
                return response;
            }
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting product",
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct Sell {
    barcode: String,
    amount: u16,
}
pub async fn sell(
    State(pool): State<Arc<Mutex<MySqlPool>>>,
    Json(payload): Json<Vec<Sell>>,
) -> impl IntoResponse {
    let pool = pool.lock().await;
    let mut transaction = match transaction::begin(&pool).await {
        Ok(transaction) => transaction,
        Err(response) => return response,
    };

    #[derive(Serialize)]
    struct Stock {
        id: u32,
        amount: u16,
        cost: BigDecimal,
    }

    let mut sold: Vec<(u16, BigDecimal)> = Vec::new();

    for product in payload {
        match sqlx::query_as!(
            Stock,
            "
            SELECT
                ID AS id,
                Amount AS amount,
                Cost AS cost
            FROM stocks
            WHERE Barcode = ?
            ",
            product.barcode
        )
        .fetch_all(&mut *transaction)
        .await
        {
            Err(e) => {
                if let Err(response) = transaction::rollback(transaction).await {
                    return response;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error selling product: {}", e),
                )
                    .into_response();
            }
            Ok(stocks) => {
                let mut remaining_amount = product.amount;

                for mut stock in stocks {
                    if stock.amount > remaining_amount {
                        sold.push((remaining_amount, stock.cost.clone()));
                        stock.amount -= remaining_amount;
                        remaining_amount = 0;
                    } else {
                        sold.push((stock.amount, stock.cost.clone()));
                        remaining_amount -= stock.amount;
                        stock.amount = 0;
                    }
                    if let Err(e) = sqlx::query!(
                        "
                            UPDATE stocks
                            SET Amount = ?
                            WHERE ID = ?;
                        ",
                        stock.amount,
                        stock.id
                    )
                    .execute(&mut *transaction)
                    .await
                    {
                        if let Err(response) = transaction::rollback(transaction).await {
                            return response;
                        }
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Error selling product: {}", e),
                        )
                            .into_response();
                    }
                }

                for each_sold in &sold {
                    if let Err(e) = sqlx::query!(
                        "
                        INSERT INTO ledger
                            (Barcode, Name, Cost, Price, Amount, Profit, VAT)
                        SELECT
                            ?, Name, ?, Price, ?, (Price / 1.07) - ?, Price - (Price / 1.07) 
                        FROM products
                        WHERE Barcode = ?;
                        ",
                        product.barcode,
                        each_sold.1,
                        each_sold.0,
                        each_sold.1,
                        product.barcode
                    )
                    .execute(&mut *transaction)
                    .await
                    {
                        if let Err(response) = transaction::rollback(transaction).await {
                            return response;
                        }
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Error selling product: {}", e),
                        )
                            .into_response();
                    }
                }
            }
        }
    }

    (axum::http::StatusCode::OK, "Product sold successfully").into_response()
}
