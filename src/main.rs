use salvo_t::test::{ResponseExt, TestClient};
use salvo_t::{prelude::*, PrintSelf};

#[tokio::main]
async fn main() {
    #[handler]
    async fn before1(
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        res.render(Text::Plain("before1"));
        if req.query::<String>("b").unwrap_or_default() == "1" {
            ctrl.skip_rest();
        } else {
            ctrl.call_next(req, depot, res).await;
        }
    }
    #[handler]
    async fn before2(
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        res.render(Text::Plain("before2"));
        if req.query::<String>("b").unwrap_or_default() == "2" {
            ctrl.skip_rest();
        } else {
            ctrl.call_next(req, depot, res).await;
        }
    }
    #[handler]
    async fn before3(
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        res.render(Text::Plain("before3"));
        if req.query::<String>("b").unwrap_or_default() == "3" {
            ctrl.skip_rest();
        } else {
            ctrl.call_next(req, depot, res).await;
        }
    }
    #[handler]
    async fn hello() -> Result<&'static str, ()> {
        Ok("hello")
    }
    let router = Router::with_path("level1").hoop(before1).push(
        Router::with_hoop(before2)
            .path("level2")
            .push(Router::with_hoop(before3).path("hello").handle(hello)),
    );
    let service = Service::new(router);

    async fn access(service: &Service, b: &str) -> String {
        TestClient::get(format!("http://127.0.0.1:7979/level1/level2/hello?b={}", b))
            .send(service)
            .await
            .take_string()
            .await
            .unwrap()
    }

    let content = access(&service, "").await;
    println!("{}",content);
    assert_eq!(content, "before1before2before3hello");
    let content = access(&service, "1").await;
    assert_eq!(content, "before1");
    let content = access(&service, "2").await;
    assert_eq!(content, "before1before2");
    let content = access(&service, "3").await;
    assert_eq!(content, "before1before2before3");
}
