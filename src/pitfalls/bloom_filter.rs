#![allow(unused)]
use crate::handler::Spell;
use crate::state::AppState;
use fred::prelude::*;
use std::error::Error;

static QUERY: &str = "
SELECT id, name, damage, created_at, updated_at
FROM spell
WHERE name = $1
";
static CREATE_QUERY: &str = "
INSERT INTO spell
(name, damage)
VALUES ($1, $2)
RETURNING (id, name, damage, created_at, updated_at)
";

// Implementing Bloom Filters for Cache Penetration

#[derive(serde::Deserialize)]
pub struct CreateSpellBody {
    pub id: i64,
    pub name: String,
    pub damage: i32,
}

pub async fn create(state: AppState, spell: CreateSpellBody) -> Result<Spell, Box<dyn Error>> {
    let mut s = state.lock().await;
    let db = &s.database;
    let spell_name = spell.name.clone();

    let spell = sqlx::query_as(CREATE_QUERY)
        .bind(&spell.name)
        .bind(&spell.damage)
        .fetch_one(db)
        .await?;

    s.bloom_filter.set(&spell_name);

    Ok(spell)
}

pub async fn find_by_name_bf(
    state: AppState,
    name: String,
) -> Result<Option<Spell>, Box<dyn Error>> {
    let mut s = state.lock().await;
    let mut tries = 100;

    if s.bloom_filter.check(&name) {
        let res: Option<Spell> = sqlx::query_as(QUERY)
            .bind(&name)
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
                    .set(spell.id, &spell, Some(Expiration::EX(60)), None, false)
                    .await;
            });

            return Ok(res);
        }
    }

    Ok(None)
}
