use anyhow::Context;
use lru_cache::LruCache;
use std::future::Future;
use std::hash::Hash;

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
    inner: LruCache<Key, Value>,
}

impl<Key, Value> DynamicCache<Key, Value>
where
    Key: Eq + Hash + ToOwned<Owned = Key>,
{
    pub fn new(
        capacity: usize,
        on_missing: Box<dyn CacheSupplier<Key, Value> + Send + Sync>,
    ) -> Self {
        Self {
            cache_supplier: on_missing,
            inner: LruCache::new(capacity),
        }
    }

    pub async fn get(&mut self, key: Key) -> anyhow::Result<&mut Value> {
        if self.inner.contains_key(&key) {
            return Ok(self.inner.get_mut(&key).unwrap());
        }

        let val = self.cache_supplier.retrieve(key.to_owned()).await?;
        self.inner.insert(key.to_owned(), val);
        self.inner.get_mut(&key).context("Critical error")
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
        let mut cache = DynamicCache::new(2, Box::new(multiplier));

        assert_eq!(*cache.get(12).await.unwrap(), 24);
        assert_eq!(*cache.get(13).await.unwrap(), 26);
        assert_eq!(*cache.get(12).await.unwrap(), 24);
        assert_eq!(*cache.get(78).await.unwrap(), 156);
        assert!(cache.get(10).await.is_err());
    }
}
