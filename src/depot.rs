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
        self.map.insert(key.into(), Box::new(value));
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::test::{ResponseExt, TestClient};
    #[test]
    fn test_depot() {
        let mut depot = Depot::with_capacity(6);
        assert!(depot.capacity() >= 6);

        depot.insert("one", "ONE".to_owned());
        assert!(depot.contains_key("one"));

        assert_eq!(depot.get::<String>("one").unwrap(), &"ONE".to_owned());
        assert_eq!(
            depot.get_mut::<String>("one").unwrap(),
            &mut "ONE".to_owned()
        );
    }

    #[test]
    fn test_transfer() {
        let mut depot = Depot::with_capacity(6);
        depot.insert("one", "ONE".to_owned());

        let depot = depot.transfer();
        assert_eq!(depot.get::<String>("one").unwrap(), &"ONE".to_owned());
    }
    #[tokio::test]
    async fn test_middleware_use_depot() {
        #[handler(internal)]
        async fn set_user(
            req: &mut Request,
            depot: &mut Depot,
            res: &mut Response,
            ctrl: &mut FlowCtrl,
        ) {
            depot.insert("user", "client");
            ctrl.call_next(req, depot, res).await;
        }
        #[handler(internal)]
        async fn hello_world(depot: &mut Depot) -> String {
            format!(
                "Hello {}",
                depot.get::<&str>("user").copied().unwrap_or_default()
            )
        }
        let router = Router::new().hoop(set_user).handle(hello_world);
        let service = Service::new(router);

        let content = TestClient::get("http://127.0.0.1:7890")
            .send(&service)
            .await
            .take_string()
            .await
            .unwrap();
        assert_eq!(content, "Hello client");
    }
}
