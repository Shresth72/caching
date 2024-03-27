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

1. The request is wait until the value is recomputed by another thread.
2. The request can immediately return a not_found response and let the client handle the situation with a back-off retry.
3. The system can maintain a stale version of the cached item to be used temporarily until the value is recomputed.

### External Computation

Use external process to recompute the value, and request workers get the value from the cache but never compute the value.
This method can be activated in various ways either proactively when a cache key is nearing expiration or reactively when a cache miss occurs.

- Disadvantage is bloating of memory as it is always the possibility of bulk of the data is never read in cache.

1. **Probabilistic Early Expiration**:

## Caching Penetration

## Caching Avalanche
