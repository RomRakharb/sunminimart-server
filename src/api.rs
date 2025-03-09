use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::database::pool;

pub async fn add_product(
    barcode: &str,
    name: &str,
    cost: f32,
    retail: f32,
    wholesale: f32,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO products
            (Barcode, Name, Cost, Retail, Wholesale, Amount)
        VALUES (?, ?, ?, ?, ?, 0);
        ",
        barcode,
        name,
        cost,
        retail,
        wholesale
    )
    .execute(pool().await)
    .await?;

    Ok(())
}

pub async fn get_price(barcode: &str) -> sqlx::Result<Option<(String, BigDecimal)>> {
    struct Product {
        pub name: String,
        pub price: BigDecimal,
    }

    let product = sqlx::query_as!(
        Product,
        "
        SELECT
            Name AS name,
            Wholesale AS price
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

pub async fn restock(barcode: &str, amount: u16, date: Option<NaiveDate>) -> sqlx::Result<()> {
    sqlx::query!(
        "
        UPDATE products
        SET Amount = Amount + ?
        WHERE Barcode = ?
        ",
        amount,
        barcode
    )
    .execute(pool().await)
    .await?;

    sqlx::query!(
        "
        INSERT INTO expire_dates
            (Barcode, Expire_date)
        VALUES (?, ?)
        ",
        barcode,
        date
    )
    .execute(pool().await)
    .await?;

    Ok(())
}

pub async fn sell(barcode: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "
        UPDATE products
        SET Amount = Amount - 1
        WHERE Barcode = ?
        ",
        barcode
    )
    .execute(pool().await)
    .await?;
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

    sqlx::query!(
        "
        DELETE FROM expire_dates
        WHERE Barcode = ?
        ",
        barcode
    )
    .execute(pool().await)
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use bigdecimal::FromPrimitive;

    use super::*;

    #[test]
    fn test_api() {
        assert!(adding_product().is_ok());
        assert!(getting_price().is_ok());
        assert!(restocking().is_ok());
        assert!(selling().is_ok());
        assert!(deleting_product().is_ok());
    }

    #[tokio::test]
    async fn adding_product() -> sqlx::Result<()> {
        add_product("0", "test", 1.0, 1.0, 1.0).await?;
        Ok(())
    }

    #[tokio::test]
    async fn getting_price() -> sqlx::Result<()> {
        if let Some((name, price)) = get_price("0").await? {
            assert_eq!(name, "test");
            assert_eq!(price, BigDecimal::from_f32(1.0).unwrap());
        };
        Ok(())
    }

    #[tokio::test]
    async fn restocking() -> sqlx::Result<()> {
        restock("0", 20, NaiveDate::from_ymd_opt(2027, 12, 16)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn selling() -> sqlx::Result<()> {
        sell("0").await?;
        Ok(())
    }

    #[tokio::test]
    async fn deleting_product() -> sqlx::Result<()> {
        delete_product("0").await?;
        Ok(())
    }
}
