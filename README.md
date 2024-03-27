# caching

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
