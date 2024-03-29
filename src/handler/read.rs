use crate::handler::Spell;
use crate::state::AppState;
use fred::prelude::*;
use std::error::Error;

static QUERY: &str = "
SELECT id, name, damage, created_at, updated_at
FROM spell
WHERE id = $1
";

pub async fn find_by_id(state: AppState, id: i64) -> Result<Option<Spell>, Box<dyn Error>> {
    let mut s = state.lock().await;

    // GET spell:id
    let cached: Option<Spell> = s.cache.get(id).await.unwrap_or(None);
    if let Some(spell) = cached {
        tracing::info!("returning cached version");
        return Ok(Some(spell));
    }

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

            // SET spell:id '{...}' [NX|XX] [EX <seconds>|PX <milliseconds>] [KEEPTTL]
            let _ = s
                .cache
                .set(id, &spell, Some(Expiration::EX(60)), None, false)
                .await;
        });
    }

    tracing::info!("returning database version");
    Ok(res)
}

/*
curl -sS localhost:3000/spells/1 | jq
*/


