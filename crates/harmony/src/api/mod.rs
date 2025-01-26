mod finance;
mod http;
mod person;
mod ping;

use std::sync::Arc;

use axum::routing;
use axum::Router;

pub fn router() -> Router {
    let state = Arc::new(http::state::StateInner::new());

    let mut router = Router::new()
        .route(ping::get::PATH, routing::get(ping::get::handler))
        .with_state(state.clone());

    router = router.merge(person::router(state.clone()));
    router = router.merge(finance::router(state.clone()));

    router = router.fallback(fallback::handler);

    router
}

mod fallback {
    use crate::api::http::prelude::*;

    #[tracing::instrument()]
    pub async fn handler() -> ResponseResult<()> {
        Ok(Response::not_found("couldn't find anything".into()))
    }
}
