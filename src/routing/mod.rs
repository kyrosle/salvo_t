use std::{borrow::Cow, collections::HashMap, fmt::format, sync::Arc};

pub mod filter;
mod router;
pub use filter::*;
pub use router::{DetectMatched, Router};

use crate::{
    depot::Depot,
    handler::Handler,
    http::{request::Request, response::Response},
};

pub type PathParams = HashMap<String, String>;

pub struct PathState {
    pub(crate) parts: Vec<String>,
    pub(crate) cursor: (usize, usize),
    pub(crate) params: PathParams,
    pub(crate) end_slash: bool,
}

impl PathState {
    pub fn new(url_path: &str) -> Self {
        let end_slash = url_path.ends_with('/');
        let parts = url_path
            .trim_start_matches('/')
            .trim_end_matches('/')
            .split('/')
            .filter_map(|p| {
                if !p.is_empty() {
                    Some(decode_url_path_safely(p))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        PathState {
            parts,
            cursor: (0, 0),
            params: PathParams::new(),
            end_slash,
        }
    }
    pub fn pick(&self) -> Option<&str> {
        match self.parts.get(self.cursor.0) {
            None => None,
            Some(part) => {
                if self.cursor.1 >= part.len() {
                    let row = self.cursor.0 + 1;
                    self.parts.get(row).map(|s| &**s)
                } else {
                    Some(&part[self.cursor.1..])
                }
            }
        }
    }
    pub fn all_rest(&self) -> Option<Cow<'_, str>> {
        if let Some(picked) = self.pick() {
            if self.cursor.0 > self.parts.len() {
                if self.end_slash {
                    Some(Cow::Owned(format!("{}/", picked)))
                } else {
                    Some(Cow::Borrowed(picked))
                }
            } else {
                let last = self.parts[self.cursor.0 + 1..].join("/");
                if self.end_slash {
                    Some(Cow::Owned(format!("{}/{}/", picked, last)))
                } else {
                    Some(Cow::Owned(format!("{}/{}", picked, last)))
                }
            }
        } else {
            None
        }
    }
    pub fn forward(&mut self, steps: usize) {
        let mut steps = steps + self.cursor.1;
        while let Some(part) = self.parts.get(self.cursor.0) {
            if part.len() > steps {
                self.cursor.1 = steps;
                return;
            } else {
                steps -= part.len();
                self.cursor = (self.cursor.0 + 1, 0);
            }
        }
    }
    pub fn ended(&self) -> bool {
        self.cursor.0 >= self.parts.len()
    }
}

fn decode_url_path_safely(path: &str) -> String {
    percent_encoding::percent_decode_str(path)
        .decode_utf8_lossy()
        .to_string()
}

pub struct FlowCtrl {
    is_ceased: bool,
    cursor: usize,
    pub(crate) handlers: Vec<Arc<dyn Handler>>,
}

impl FlowCtrl {
    pub fn new(handlers: Vec<Arc<dyn Handler>>) -> Self {
        FlowCtrl {
            is_ceased: false,
            cursor: 0,
            handlers,
        }
    }
    pub fn has_next(&self) -> bool {
        self.cursor < self.handlers.len() && !self.handlers.is_empty()
    }
    pub async fn call_next(
        &mut self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
    ) -> bool {
        if res.is_stamped() {
            self.skip_rest();
            return false;
        }
        let mut handler = self.handlers.get(self.cursor).cloned();
        if handler.is_none() {
            false
        } else {
            while let Some(h) = handler.take() {
                self.cursor += 1;
                h.handle(req, depot, res, self).await;
                if res.is_stamped() {
                    self.skip_rest();
                    return true;
                } else {
                    handler = self.handlers.get(self.cursor).cloned();
                }
            }
            true
        }
    }
    pub fn skip_rest(&mut self) {
        self.cursor = self.handlers.len()
    }
    pub fn is_ceased(&self) -> bool {
        self.is_ceased
    }
    pub fn cease(&mut self) {
        self.skip_rest();
        self.is_ceased = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::test::{ResponseExt, TestClient};

    #[tokio::test]
    #[ignore]
    async fn test_custom_filter() {
        #[handler(internal)]
        async fn hello_world() -> &'static str {
            "Hello World"
        }

        let router = Router::new()
            .filter_fn(|req, _| {
                let host = req.uri().host().unwrap_or_default();
                host == "localhost"
            })
            .get(hello_world);
        let service = Service::new(router);

        async fn access(service: &Service, host: &str) -> String {
            TestClient::get(format!("http://{}/", host))
                .send(service)
                .await
                .take_string()
                .await
                .unwrap()
        }

        assert!(access(&service, "127.0.0.1").await.contains("404: Not Found"));
        assert_eq!(access(&service, "localhost").await, "Hello World");
    }
}
