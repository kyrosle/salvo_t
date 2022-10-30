use async_trait::async_trait;
use hyper::StatusCode;

use crate::{
    depot::Depot,
    http::{request::Request, response::Response},
    routing::FlowCtrl,
};

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    #[must_use = "handle future must be used"]
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    );
}

#[allow(non_camel_case_types)]
pub struct empty_handler;
#[async_trait]
impl Handler for empty_handler {
    async fn handle(
        &self,
        _req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        res.set_status_code(StatusCode::OK);
    }
}

pub trait Skipper: Send + Sync + 'static {
    fn skipped(&self, req: &mut Request, depot: &Depot) -> bool;
}
impl<F> Skipper for F
where
    F: Fn(&mut Request, &Depot) -> bool + Send + Sync + 'static,
{
    fn skipped(&self, req: &mut Request, depot: &Depot) -> bool {
        (self)(req, depot)
    }
}

pub fn none_skipper(_req: &mut Request, _depot: &Depot) -> bool {
    false
}

macro_rules! handler_tuple_impls {
    (
        $(
            $Tuple:tt {
                $(($idx:tt) -> $T:ident,)+
            }
        )+
    ) => {
        $(
            #[async_trait::async_trait]
            impl<$($T,)+> Handler for ($($T,)+) where $($T: Handler,)+
            {
                async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl)
                {
                    $(
                        if !res.is_stamped() {
                            self.$idx.handle(req, depot, res, ctrl).await;
                        }
                    )+
                }
            }
        )+
    };
}

macro_rules! skipper_tuple_impls {
    (
        $(
            $Tuple:tt {
                $(($idx:tt) -> $T:ident,)+
            }
        )+
    ) => {
        $(
            impl<$($T,)+> Skipper for ($($T,)+) where $($T: Skipper,)+
            {
                fn skipped(&self, req: &mut Request, depot: &Depot) -> bool {
                    $(
                        if self.$idx.skipped(req, depot) {
                            return true;
                        }
                    )+
                    false
                }
            }
        )+
    };
}

macro_rules! __for_each_tuple {
    ($callback:ident) => {
        $callback! {
            1 {
                (0) -> A,
            }
            2 {
                (0) -> A,
                (1) -> B,
            }
            3 {
                (0) -> A,
                (1) -> B,
                (2) -> C,
            }
            4 {
                (0) -> A,
                (1) -> B,
                (2) -> C,
                (3) -> D,
            }
            5 {
                (0) -> A,
                (1) -> B,
                (2) -> C,
                (3) -> D,
                (4) -> E,
            }
            6 {
                (0) -> A,
                (1) -> B,
                (2) -> C,
                (3) -> D,
                (4) -> E,
                (5) -> F,
            }
            7 {
                (0) -> A,
                (1) -> B,
                (2) -> C,
                (3) -> D,
                (4) -> E,
                (5) -> F,
                (6) -> G,
            }
            8 {
                (0) -> A,
                (1) -> B,
                (2) -> C,
                (3) -> D,
                (4) -> E,
                (5) -> F,
                (6) -> G,
                (7) -> H,
            }
        }
    };
}

__for_each_tuple!(handler_tuple_impls);
__for_each_tuple!(skipper_tuple_impls);