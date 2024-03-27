#![allow(unused)]
use crate::handler::Spell;
use crate::state::AppState;
use fred::prelude::*;
use std::error::Error;

static QUERY: &str = "
SELECT id, name, damage, created_at, updated_at
FROM spell
WHERE id = $1
";

// Implementing Cache Locks & Retry for Cache Stampedes
// (only useful for high traffic applications)
pub async fn find_by_id_cs(state: AppState, id: i64) -> Result<Option<Spell>, Box<dyn Error>> {
    let mut s = state.lock().await;
    let mut tries = 100;

    loop {
        let cached = s.cache.get(id).await.unwrap_or(None);
        if let Some(spell) = cached {
            tracing::info!("returning cached version");
            return Ok(Some(spell));
        }

        if s.cache.add_lock(id).await? || tries == 0 {
            let res: Option<Spell> = sqlx::query_as(QUERY)
                .bind(id)
                .fetch_optional(&s.database)
                .await?;

            if let Some(spell) = &res {
                let spell = spell.clone();
                let state = state.clone();

                tokio::spawn(async move {
                    let mut s = state.lock().await;

                    tracing::info!("caching spell");
                    let _ = s
                        .cache
                        .set(id, &spell, Some(Expiration::EX(60)), None, false)
                        .await;

                    let _ = s.cache.del_lock(id).await;
                });
            }

            tracing::info!("returning database version");
            return Ok(res);
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            tries -= 1;
        }
    }
}
