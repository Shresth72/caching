#![allow(unused)]

use super::Spell;
use crate::state::AppState;
use sqlx::{Acquire, Postgres, Transaction};
use std::error::Error;

pub async fn perform_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    id: i64,
) -> Result<Option<Spell>, Box<dyn Error>> {
    let _: Option<Spell> = sqlx::query_as(
        r#"
        INSERT INTO spell (name, damage) VALUES ($1, $2)
        "#,
    )
    .bind("Psychic")
    .bind(10)
    .fetch_optional(&mut **transaction)
    .await?;

    let res: Option<Spell> = sqlx::query_as(
        r#"
        SELECT id, name, damage, created_at, updated_at
        FROM spell
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(res)
}

pub async fn call_transaction(state: AppState, id: i64) -> Result<Option<Spell>, Box<dyn Error>> {
    let s = state.lock().await;
    let mut lock = s.database.acquire().await?;
    let mut transaction = lock.begin().await?;

    let res = perform_transaction(&mut transaction, id).await?;

    transaction.commit().await?;
    // transaction.rollback().await?;

    Ok(res)
}
