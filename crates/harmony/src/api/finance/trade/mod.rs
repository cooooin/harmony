mod transaction;

use crate::api::http::state::StateInner;

pub fn router(state: std::sync::Arc<StateInner>) -> axum::Router {
    use axum::routing::{delete, get, post, put};

    let mut router = axum::Router::new()
        .route(get::PATH, get(get::handler))
        .route(post::PATH, post(post::handler))
        .route(put::PATH, put(put::handler))
        .route(delete::PATH, delete(delete::handler))
        .with_state(state.clone());

    router = router.merge(transaction::router(state));

    router
}

mod get {
    pub const PATH: &str = "/finance/trades";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::paginate;
    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::Trade;

    #[derive(Debug, Validate, Deserialize)]
    pub struct Params {
        pub id: Option<i64>,
        #[validate(range(min = 1))]
        pub page: Option<usize>,
        #[validate(range(min = 1, max = 1024))]
        pub page_size: Option<usize>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TradeItem {
        pub id: i64,
        pub owner: i64,
        pub base_object_id: i64,
        pub quote_object_id: i64,
        pub alias: Option<String>,
        pub remark: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub trades: Vec<TradeItem>,
        pub total: usize,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Query(params): Query<Params>,
    ) -> ResponseResult<ResponseBody> {
        params.validate()?;

        let owner = claim.subject();
        let conn = connection()?;

        let total = Trade::count_by_owner(&conn, owner)?;

        if let Some(id) = params.id {
            let trade = Trade::select_by_id_owner(&conn, id, owner)?
                .ok_or(Response::not_found(format!("trade {} does not exist", id)))?;

            let trade_item = TradeItem {
                id: trade.id(),
                owner: trade.owner,
                base_object_id: trade.base_object_id,
                quote_object_id: trade.quote_object_id,
                alias: trade.alias,
                remark: trade.remark,
                created_at: trade.created_at,
                updated_at: trade.updated_at,
            };

            return Ok(Response::ok(ResponseBody {
                trades: vec![trade_item],
                total,
            }));
        }

        let (limit, offset) = paginate(params.page.unwrap_or(1), params.page_size.unwrap_or(256));

        let trades = Trade::select_by_owner(&conn, owner, limit, offset)?;

        let trades = trades
            .into_iter()
            .map(|trade| TradeItem {
                id: trade.id(),
                owner: trade.owner,
                base_object_id: trade.base_object_id,
                quote_object_id: trade.quote_object_id,
                alias: trade.alias,
                remark: trade.remark,
                created_at: trade.created_at,
                updated_at: trade.updated_at,
            })
            .collect();

        Ok(Response::ok(ResponseBody { trades, total }))
    }
}

mod post {
    pub const PATH: &str = "/finance/trades";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::object::Object;
    use crate::model::finance::trade::Trade;

    #[derive(Debug, Clone, Validate, Serialize, Deserialize)]
    pub struct RequestBody {
        #[validate(range(min = 1))]
        pub base_object_id: i64,
        #[validate(range(min = 1))]
        pub quote_object_id: i64,
        #[validate(length(min = 1, max = 4096))]
        pub alias: Option<String>,
        #[validate(length(min = 1, max = 4096))]
        pub remark: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub id: i64,
        pub created_at: DateTime<Utc>,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Json(payload): Json<RequestBody>,
    ) -> ResponseResult<ResponseBody> {
        payload.validate()?;

        let owner = claim.subject();
        let conn = connection()?;

        let base_object = Object::select_by_id_owner(&conn, payload.base_object_id, owner)?.ok_or(
            Response::not_found(format!("object {} does not exist", payload.base_object_id)),
        )?;

        let quote_object =
            Object::select_by_id_owner(&conn, payload.quote_object_id, owner)?.ok_or(
                Response::not_found(format!("object {} does not exist", payload.quote_object_id)),
            )?;

        let id = Trade::insert(
            &conn,
            owner,
            base_object.id(),
            quote_object.id(),
            payload.alias,
            payload.remark,
        )?;

        let created_at = Utc::now();

        Ok(Response::ok(ResponseBody { id, created_at }))
    }
}

mod put {
    pub const PATH: &str = "/finance/trades/:id";

    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::Trade;

    #[derive(Debug, Clone, Validate, Serialize, Deserialize)]
    pub struct RequestBody {
        #[validate(range(min = 1))]
        pub base_object_id: Option<i64>,
        #[validate(range(min = 1))]
        pub quote_object_id: Option<i64>,
        #[validate(length(min = 1, max = 4096))]
        pub alias: Option<String>,
        #[validate(length(min = 1, max = 4096))]
        pub remark: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub id: i64,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Path(id): Path<i64>,
        Json(payload): Json<RequestBody>,
    ) -> ResponseResult<ResponseBody> {
        payload.validate()?;

        let owner = claim.subject();
        let conn = connection()?;

        let trade = Trade::select_by_id_owner(&conn, id, owner)?
            .ok_or(Response::not_found(format!("trade {} does not exist", id)))?;

        let base_object_id = payload.base_object_id.unwrap_or(trade.base_object_id);
        let quote_object_id = payload.quote_object_id.unwrap_or(trade.quote_object_id);
        let alias = payload.alias.or(trade.alias);
        let remark = payload.remark.or(trade.remark);

        Trade::update_by_id_owner(
            &conn,
            id,
            owner,
            base_object_id,
            quote_object_id,
            alias,
            remark,
        )?;

        Ok(Response::ok(ResponseBody { id }))
    }
}

mod delete {
    pub const PATH: &str = "/finance/trades/:id";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::Trade;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TradeItem {
        pub id: i64,
        pub owner: i64,
        pub base_object_id: i64,
        pub quote_object_id: i64,
        pub alias: Option<String>,
        pub remark: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub id: i64,
    }

    #[tracing::instrument()]
    pub async fn handler(claim: Claim, Path(id): Path<i64>) -> ResponseResult<TradeItem> {
        let owner = claim.subject();
        let conn = connection()?;

        let trade = Trade::select_by_id_owner(&conn, id, owner)?
            .ok_or(Response::not_found(format!("trade {} does not exist", id)))?;

        Trade::delete_by_id_owner(&conn, id, owner)?;

        let trade_item = TradeItem {
            id: trade.id(),
            owner: trade.owner,
            base_object_id: trade.base_object_id,
            quote_object_id: trade.quote_object_id,
            alias: trade.alias,
            remark: trade.remark,
            created_at: trade.created_at,
            updated_at: trade.updated_at,
        };

        Ok(Response::ok(trade_item))
    }
}
