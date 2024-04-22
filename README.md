# Caching - Cache Aside Pattern & Caching Pitfalls

# Caching Concepts

## Cache Aside Pattern

Cache Aside Pattern is widely used for cache read/write patterns and allows for the system to be aware of all the processes that occur on the Database and the Cache.

Both Read/Write patterns have steps that are:

### Read

- Return the data from the Cache if it exists
- If not, return it from the Database and update it to the Cache.

```rust
pub async fn find_by_id_2(state: AppState, id: i64) -> Result<Option<Spell>, Box<dyn Error>> {
    let mut s = state.lock().await;

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
        tracing::info!("caching spell");

        let _ = s
            .cache
            .set(id, &spell, Some(Expiration::EX(60)), None, false)
            .await;
    }

    tracing::info!("returning database version");
    Ok(res)
}
```

### Write

- Update the Database
- Delete Cache

```rust
pub async fn update(
    state: AppState,
    id: i64,
    body: UpdateBody,
) -> Result<Option<Spell>, Box<dyn Error>> {
    tracing::info!("updating spell: {}", id);
    let mut s = state.lock().await;

    // update db
    let res: Option<Spell> = sqlx::query_as(QUERY)
        .bind(body.damage)
        .bind(id)
        .fetch_optional(&s.database)
        .await?;

    // delete cache
    s.cache.del(id).await?;

    todo!()
}
```

#### So, why did we delete the cache after the update and not update it instead?

- From the Cache Aside Pattern, the cache is only updated when the data is read from the database. This is because the cache is not the source of truth, the database is.
- Also, updating the cache after the database is updated can lead to performance and security issues.
  
Following are the reasons:

#### Performance & Cache Perturbation

- For large cache items, updating the cache every single time can be be expensive.
- There may also be case where the cache is not used at all, and updating the cache every time is a waste of resources known as Cache Perturbation.
- However, if the service is read heavy, then updating the cache might be beneficial.

#### Security

- In case of concurrent updates, multiple writes can lead to data inconsistency.
- Let's consider a scenario:
  - Write Request 1 and Write Request 2 are updating the same data subsequently in the cache.
  - Due to thread scheduling, Write Request 2 operation happens to be executed before Write Request 1.
  - This leads to the cache being updated with the data from Write Request 2, so the data from Write request is written to the cache.
  - Due to this, later operations will read the old data.

#### Theoretically, Cache Aside Pattern still may cause data inconsistencies, let's consider this process

- Similar to the above scenario, Delete operation of Write Request 1 may be executed after the Write operation of Write Request 2.
- This leads to cache data lagging behind the database data.
- However, the probability of this happening is much lower, as the cache is updated only when the data is read from the database. Scheduling time for database is much higher than the cache.

<!-- ### Can Cache Aside Pattern completely prevent data inconsistencies?

-  --> 

## Caching Invalidation

We can invalidate cache in two ways:

1. **Time-based Invalidation**: We can set a time limit for the cache. After the time limit, the cache will be invalidated. This is the simplest way to invalidate cache. But it is not the best way to invalidate cache. Because, if the cache is invalidated after a certain time, then the cache will be invalidated.

```rust
let _ = s
  .cache
    .set(
      id,
      &spell,
      Some(Expiration::EX(60)),
      None,
      false,
    ).await;
```

1. **Event-based Invalidation**: We can invalidate cache based on the event. When the event occurs, we can invalidate the cache. This is the best way to invalidate cache. Because, we can invalidate the cache when the event occurs.

```rust
tracing::info!("deleting cached spell");
let _ = s.cache.del(id).await;
```

# Caching Pitfalls

## Caching Stampede

Cache Stampede is a problem that occurs when a resource heavy cache item/page is invalidated. When the cache is invalidated, all the requests will be sent to the database. This will cause the database to be overloaded. This also prevents the pages to be recached.

To solve this problem, we can use the following methods:

### Locking

Upon a cache miss, the first request to the cache will lock the cache and fetch the data from the database. The subsequent requests will wait for the first request to finish and then return the data from the cache.

However, this locking can fail. So, we can use the following methods:

1. The request waits until the value is recomputed by another thread.
2. The request can immediately return a not_found response and let the client handle the situation with a back-off retry.
3. The system can maintain a stale version of the cached item to be used temporarily until the value is recomputed.

```rust
// prevents cache stampede using backoff retry and cache lock
// (only useful for high traffic applications)
pub async fn find_by_id_cs(state: AppState, id: i64) -> Result<Option<Spell>, Box<dyn Error>> {
    let mut s = state.lock().await;
    let mut tries = 100;

    // loops except when for first request or tries reaches 0
    loop {
        let cached = s.cache.get(id).await.unwrap_or(None);
        if let Some(spell) = cached {
            tracing::info!("returning cached version");
            return Ok(Some(spell));
        }

        // create a cache lock
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
            // lock was not available for the cache id
            // it fallsback and retries after 25ms
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            tries -= 1;
        }
    }
}
```

### External Computation

Use external process to recompute the value, and request workers get the value from the cache but never compute the value.
This method can be activated in various ways either proactively when a cache key is nearing expiration or reactively when a cache miss occurs.

- Disadvantage is bloating of memory as it is always the possibility of bulk of the data is never read in cache.

### Probabilistic Early Expiration

Each request has a small chance of proactively triggering a computation of the value before the expiration time. The likelihood increases as the expiration time approaches. This method is useful when the cost of re-computation is low.

## Caching Penetration

This problem happens when a request is made for a non-existent database cache item. This results in unnecessary requests to the database.

### Approach 1

- To mitigate cache penetration, implement a placeholder value for non-existent keys. This way the data hits the placeholders in cache instead of pointless database requests again and again.
- There should be appropriate TTLs for the placeholder values to prevent cache from being bloated. However, the placeholder values should be set to expire after a longer period than the actual cache values but requires careful tuning to avoid significant resource consumption.

### Bloom Filter

- It is a space efficient probabilistic data structure that is used to test whether an element is a member of a set. It is used to reduce the number of requests to the database by filtering out requests that are not in the cache.
- When new records are added to storage, their keys are recorded in blood filter. Before fetching records, the application checks the bloom filter first. If the key is not in the bloom filter, the application does not query the database.
- Small percentage of cache misses may still occur due to false positives in the bloom filter. However, false negatives are not possible.

```rust
let bloom_filter: Bloom<String> = Bloom::new_for_fp_rate(1000, 0.001);
s.bloom_filter.set(&spell_name);

let exists = s.bloom_filter.check(&spell_name);
```

#### Other Features and Working

- Many NoSQL databases use Blood Filters to reduce disk space and reads for keys that don't exist. LSM tress are expensive and take time to read from disk. Blood Filters are used to check if the key exists in the database or not.
- Web Browsers use bloom filters to track all the URLS seen and only cache a page on the second request. This reduces the caching workload significantly and increases the caching hit rate. Also, used in detecting malicious URLs.
- A critical part of good bloom filter is the hash function. The hash function should be:
  - Fast
  - Outputs should be evenly and randomly distributed
  - Collision may occur rarely
- Hash functions set values for the key for be set to 1 in the bloom filter buckets. The number of hash functions is determined by the size of the bloom filter and the number of elements to be stored in the bloom filter.

## Caching Avalanche

Cache Avalanche is when a significant number of caches expires at the same time or the cache restart and is empty. This leads to a large number of requests tht don't hit the cache and instead directly query the database. This puts a lot of pressure on the database and can potentially cause it to crash.
It's different from cache stampedes, as in cache stampede the servers try to refresh singular data points in when a failure happens. Whereas, cache avalance is a much broader issue.

Solutions:

### Circuit Breaker

- Implement a Circuit Breaker, that temporarily blocks imcoming requests when the system is clearly overloaded. This prevents total meltdown and buys time for recovery.

### Cache Replicas

- Deploy highly available cache clusters with redundancy, if parts of the cache go down, other parts remain operational. Hence, reducing the severity of full crashes.

### Randomized Expiration Times

- When implementing Cache Pattern, expiration times should be set for cache invalidation and preventing the cache to bloat all the memeory, along with a random number within the set cache expiration time. Hence, some caches will expire later, sharing some of the pressure of loading data into the DB.
- The random time range should be based on the business logic of the service, and should be dispersed, to reduce the probability of cache avalanches happening.

### Cache Prewarming

- Used when a cache restarts, so the essential data is proactivly populated in the cold cache before it's put into service. Hence, avoid too many requests to db later on.

However, Caching should generally be the last resort, after you've tried tuning and optimizing your queries, indexing etc. Databases with indexing or read replicas generally perform better as caching may add a lot of overhead if not done properly.
