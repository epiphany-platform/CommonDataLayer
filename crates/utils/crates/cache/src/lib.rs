use lru::LruCache;
use std::future::Future;
use std::hash::Hash;
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait CacheSupplier<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    async fn retrieve(&self, key: Key) -> anyhow::Result<Value>;
}

#[async_trait::async_trait]
impl<Key, Value, Fun, Fut> CacheSupplier<Key, Value> for Fun
where
    Fun: (Fn(Key) -> Fut) + Sync + Send,
    Fut: Future<Output = anyhow::Result<Value>> + Sync + Send,
    Key: Eq + Hash + ToOwned<Owned = Key> + Sync + Send + 'static,
{
    async fn retrieve(&self, key: Key) -> anyhow::Result<Value> {
        (*self)(key).await
    }
}

pub struct DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    cache_supplier: Box<dyn CacheSupplier<Key, Value> + Send + Sync>,
    inner: Mutex<LruCache<Key, Value>>,
}

impl<Key, Value> DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
    Value: Clone,
{
    pub fn new(
        capacity: usize,
        on_missing: Box<dyn CacheSupplier<Key, Value> + Send + Sync>,
    ) -> Self {
        Self {
            cache_supplier: on_missing,
            inner: Mutex::new(LruCache::new(capacity)),
        }
    }

    pub async fn get(&self, key: Key) -> anyhow::Result<Value> {
        {
            let mut cache = self.inner.lock().await;
            if cache.contains(&key) {
                return Ok(cache.get(&key).unwrap().clone());
            }
        }

        let value = self.cache_supplier.retrieve(key.to_owned()).await?;

        let mut cache = self.inner.lock().await;
        // This check is mandatory, as we aren't sure if other process didn't update cache before us
        if !cache.contains(&key) {
            cache.put(key.to_owned(), value.clone());
            Ok(value)
        } else {
            Ok(cache.get(&key).unwrap().clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DynamicCache;

    async fn multiplier(key: i32) -> anyhow::Result<i32> {
        if key == 10 {
            anyhow::bail!("Can't fetch 10");
        }
        Ok(key * 2)
    }

    #[tokio::test]
    async fn computes_new_values() {
        let cache = DynamicCache::new(2, Box::new(multiplier));

        assert_eq!(cache.get(12).await.unwrap(), 24);
        assert_eq!(cache.get(13).await.unwrap(), 26);
        assert_eq!(cache.get(12).await.unwrap(), 24);
        assert_eq!(cache.get(78).await.unwrap(), 156);
        assert!(cache.get(10).await.is_err());
    }
}
