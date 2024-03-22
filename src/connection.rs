use crate::state;

use dotenv::dotenv;
use fred::prelude::*;
use sqlx::postgres::PgPoolOptions;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn conn() -> Result<Arc<Mutex<state::StateInternal>>, Box<dyn Error>> {
    dotenv()?;

    let pg_url = std::env::var("DATABASE_URL")?;
    let redis_url = match std::env::var("REDIS_URL")?.as_str() {
        "" => "redis://localhost:5432".to_string(),
        x => x.to_string(),
    };

    let dbpool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&pg_url)
        .await?;

    // sqlx::migrate!().run(&dbpool).await?;

    let pool_size = 8;
    let config = RedisConfig::from_url(&redis_url)?;

    let redis_pool = Builder::from_config(config)
        .with_performance_config(|config| {
            config.auto_pipeline = true;
        })
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(pool_size)
        .expect("Failed to create redis pool");

    if std::env::var("REDIS_URL")? != "" {
        redis_pool.init().await.expect("Failed to connect to redis");
        // let _ = redis_pool.flushall::<i32>(false).await;
    }

    Ok(Arc::new(Mutex::new(state::StateInternal::new(
        dbpool, redis_pool,
    ))))
}
