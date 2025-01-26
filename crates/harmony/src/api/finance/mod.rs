mod object;
mod trade;

use std::sync::Arc;

use axum::Router;

use crate::api::http::state::StateInner;

pub fn router(state: Arc<StateInner>) -> Router {
    let mut router = Router::new().with_state(state.clone());

    router = router.merge(object::router(state.clone()));
    router = router.merge(trade::router(state.clone()));

    router
}
