use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
};

#[derive(Default)]
pub struct Depot {
    map: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Depot {
    pub fn new() -> Depot {
        Depot {
            map: HashMap::new(),
        }
    }
    pub fn inner(&self) -> &HashMap<String, Box<dyn Any + Send + Sync>> {
        &self.map
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Depot {
            map: HashMap::with_capacity(capacity),
        }
    }
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
    pub fn inject<V: Any + Send + Sync>(&mut self, value: V) -> &mut Self {
        self.map
            .insert(format!("{:?}", TypeId::of::<V>()), Box::new(value));
        self
    }
    #[allow(unconditional_recursion)]
    pub fn insert<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Any + Send + Sync,
    {
        self.insert(key.into(), Box::new(value));
        self
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    pub fn get<V: Any + Send + Sync>(&self, key: &str) -> Option<&V> {
        self.map.get(key).and_then(|b| b.downcast_ref::<V>())
    }
    pub fn get_mut<V: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut V> {
        self.map.get_mut(key).and_then(|b| b.downcast_mut::<V>())
    }
    pub fn remove<V: Any + Send + Sync>(&mut self, key: &str) -> Option<V> {
        self.map
            .remove(key)
            .and_then(|b| b.downcast::<V>().ok())
            .map(|b| *b)
    }
    pub fn transfer(&mut self) -> Self {
        let mut map = HashMap::with_capacity(self.map.len());
        for (k, v) in self.map.drain() {
            map.insert(k, v);
        }
        Self { map }
    }
}

impl fmt::Debug for Depot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Depot")
            .field("keys", &self.map.keys())
            .finish()
    }
}

// TODO: depot.rs test mod
// #[cfg(test)]
// mod test {
// }
