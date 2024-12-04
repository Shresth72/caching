#![allow(unused)]

use crate::handler::Spell;
use crate::state::AppState;
use rand::{thread_rng, Rng};
use std::error::Error;

#[derive(serde::Deserialize)]
pub struct CreateSpellBody {
    pub name: String,
    pub damage: i32,
}

static QUERY: &str = r#"INSERT INTO spell (id, name, damage) VALUES ($1, $2, $3) RETURNING id, name, damage, created_at, updated_at;"#;

pub async fn create(state: AppState, spell: CreateSpellBody) -> Result<Spell, Box<dyn Error>> {
    let db = &state.lock().await.database;

    let spell_id: i64 = thread_rng().gen();
    let spell = sqlx::query_as(QUERY)
        .bind(spell_id)
        .bind(&spell.name)
        .bind(&spell.damage)
        .fetch_one(db)
        .await?;

    Ok(spell)
}
