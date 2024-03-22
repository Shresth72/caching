use crate::state::AppState;
use std::error::Error;

static QUERY: &str = "
DELETE FROM spell WHERE id = $1
";

pub async fn delete_spell(
    state: AppState, id: i64,
) -> Result<u64, Box<dyn Error>> {
    tracing::info!("deleting spell: {}", id);

    let mut s = state.lock().await;

    if let Some(_) = s.cache.get(id).await? {
        let state = state.clone();

        tokio::spawn(async move {
            let mut s = state.lock().await;

            tracing::info!("deleting cached spell");
            let _ = s.cache.del(id).await;
        });
    }

    let res = sqlx::query(QUERY)
        .bind(id)
        .execute(&s.database).await?;

    Ok(res.rows_affected())
}

/*
curl -X DELETE localhost:3000/spells/1
*/