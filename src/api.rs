use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::database::pool;

pub async fn add_product(barcode: &str, name: &str, price: u16) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO products
            (Barcode, Name, Price)
        VALUES (?, ?, ?);
        ",
        barcode,
        name,
        price
    )
    .execute(pool().await)
    .await?;

    Ok(())
}

pub async fn get_price(barcode: &str) -> sqlx::Result<Option<(String, u16)>> {
    struct Product {
        pub name: String,
        pub price: u16,
    }

    let product = sqlx::query_as!(
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
    .await?;

    if let Some(product) = product {
        Ok(Some((product.name, product.price)))
    } else {
        Ok(None)
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
