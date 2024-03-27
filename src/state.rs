#![allow(dead_code)]
use crate::handler::Spell;
use fred::interfaces::KeysInterface;
use fred::{clients::RedisPool, prelude::*};
use serde_json::Value;
use sqlx::postgres::PgPool;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type AppState = Arc<Mutex<StateInternal>>;

pub struct StateInternal {
    pub database: sqlx::postgres::PgPool,
    pub cache: Cache,
}

impl StateInternal {
    pub fn new(db: PgPool, redis: RedisPool) -> Self {
        StateInternal {
            database: db,
            cache: Cache { internal: redis },
        }
    }
}

pub struct Cache {
    internal: RedisPool,
}

impl Cache {
    fn key_for_id(id: i64) -> String {
        format!("spell:{}", id)
    }

    fn key_for_lock(id: i64) -> String {
        format!("spell:lock:{}", id)
    }

    pub async fn get(&mut self, id: i64) -> Result<Option<Spell>, Box<dyn Error>> {
        if !self.internal.is_connected() {
            return Err(Box::new(simple_error::SimpleError::new(
                "not connected redis",
            )));
        }

        let value: Option<Value> = self.internal.get(Self::key_for_id(id)).await?;

        let spell = match value {
            Some(x) => match serde_json::from_value(x) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            None => None,
        };
        Ok(spell)
    }

    pub async fn set(
        &mut self,
        id: i64,
        spell: &Spell,
        expiration: Option<Expiration>,
        set_opts: Option<SetOptions>,
        get: bool,
    ) -> Result<(), Box<dyn Error>> {
        if !self.internal.is_connected() {
            return Err(Box::new(simple_error::SimpleError::new(
                "not connected redis",
            )));
        }

        let value: Value = serde_json::to_value(spell)?;
        let key = Self::key_for_id(id);
        self.internal
            .set(key, value.to_string(), expiration, set_opts, get)
            .await?;
        Ok(())
    }

    pub async fn del(&mut self, id: i64) -> Result<(), Box<dyn Error>> {
        let key = Self::key_for_id(id);
        self.internal.del(key).await?;
        Ok(())
    }

    pub async fn add_lock(&mut self, id: i64) -> Result<bool, Box<dyn Error>> {
        let key = Self::key_for_lock(id);
        let cached_lock: Option<Value> = self.internal.get(&key).await.unwrap_or(None);
        if let Some(_) = cached_lock {
            return Ok(false);
        }

        self.internal
            .set(key, "1".to_string(), None, None, false)
            .await?;

        Ok(true)
    }

    pub async fn del_lock(&mut self, id: i64) -> Result<(), Box<dyn Error>> {
        let key = Self::key_for_lock(id);
        self.internal.del(key).await?;
        Ok(())
    }
}
