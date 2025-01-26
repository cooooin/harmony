pub mod get {
    pub const PATH: &str = "/ping";

    use serde::{Deserialize, Serialize};

    use crate::api::http::prelude::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub timestamp: u128,
    }

    #[tracing::instrument(skip(state))]
    pub async fn handler(state: State) -> Response<ResponseBody> {
        Response::ok(ResponseBody {
            timestamp: state.timestamp_millis(),
        })
    }
}
