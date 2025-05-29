use crate::collections::hash::RandomState;

pub struct HashMap<K, V> {
    inner: hashbrown::HashMap<K, V, RandomState>,
}

impl<K, V> HashMap<K, V>
where
    K: core::hash::Hash + Eq,
{
    pub fn new() -> Self {
        HashMap {
            inner: hashbrown::HashMap::with_hasher(RandomState),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }
}
