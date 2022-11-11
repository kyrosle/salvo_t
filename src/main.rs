use salvo_t::prelude::*;
use salvo_t::routing::PathState;
use salvo_t::test::*;

#[tokio::main]
async fn main() {
    #[handler]
    async fn fake_handler(_res: &mut Response) {}
    let router = Router::default().push(
        Router::with_path("users")
            .push(Router::with_path("<id>").push(Router::with_path("emails").get(fake_handler))),
    );
    dbg!("{:?}", &router);
    let mut req = TestClient::get("http://local.host/users/12/emails").build();
    dbg!("{:?}", &req.uri());
    let mut path_state = PathState::new(req.uri().path());
    let matched = router.detect(&mut req, &mut path_state);
}
