use axum::{
    extract::{self, Path},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDate;
use sqlx::types::BigDecimal;
use sqlx::types::Decimal;
use tokio::task::JoinSet;

use std::str::FromStr;

use crate::database::pool;

#[derive(Deserialize)]
pub struct AddProduct {
    barcode: String,
    name: String,
    cost: Decimal,
    price: u16,
    amount: u16,
    expire_dates: Vec<[u32; 3]>, // ymd
}
pub async fn add_product(extract::Json(payload): extract::Json<AddProduct>) -> impl IntoResponse {
    let mut set = JoinSet::new();

    set.spawn(
        sqlx::query!(
            "
            INSERT INTO products
                (Barcode, Name, Price)
            VALUES (?, ?, ?);
            ",
            payload.barcode,
            payload.name,
            payload.price
        )
        .execute(pool().await),
    );

    set.spawn(
        sqlx::query!(
            "
            INSERT INTO stocks
                (Barcode, Cost, amount)
            VALUES (?, ?, ?);
            ",
            payload.barcode,
            payload.cost,
            payload.amount
        )
        .execute(pool().await),
    );

    for exp in payload.expire_dates {
        if let Some(date) = NaiveDate::from_ymd_opt(exp[0] as i32, exp[1], exp[2]) {
            set.spawn(
                sqlx::query!(
                    "
            INSERT INTO expire_dates
                (Barcode, ExpireDate)
            VALUES (?, ?);
            ",
                    payload.barcode,
                    date
                )
                .execute(pool().await),
            );
        } else {
            set.spawn(async { Err(sqlx::Error::Decode("Invalid expire date".into())) });
        }
    }

    if set.join_all().await.iter().all(|res| res.is_ok()) {
        (axum::http::StatusCode::OK, "Product added successfully").into_response()
    } else {
        let _ = sqlx::query!(
            "
            DELETE FROM products WHERE Barcode = ?;
            ",
            payload.barcode,
        )
        .execute(pool().await)
        .await;
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
