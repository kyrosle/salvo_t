use salvo_t::prelude::*;
use salvo_t::test::*;

#[tokio::main]
async fn main() {
    #[handler]
    async fn set_user(
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        depot.insert("user", "client");
        ctrl.call_next(req, depot, res).await;
    }
    #[handler]
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
}
