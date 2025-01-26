use crate::api::http::state::StateInner;

pub fn router(state: std::sync::Arc<StateInner>) -> axum::Router {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route(get::PATH, get(get::handler))
        .route(post::PATH, post(post::handler))
        .route(put::PATH, put(put::handler))
        .route(delete::PATH, delete(delete::handler))
        .with_state(state)
}

mod get {
    pub const PATH: &str = "/finance/objects";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::paginate;
    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::object::Object;

    #[derive(Debug, Validate, Deserialize)]
    pub struct Params {
        pub id: Option<i64>,
        #[validate(range(min = 1))]
        pub page: Option<usize>,
        #[validate(range(min = 1, max = 1024))]
        pub page_size: Option<usize>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ObjectItem {
        pub id: i64,
        pub owner: i64,
        pub symbol: String,
        pub alias: Option<String>,
        pub remark: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub objects: Vec<ObjectItem>,
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

        let total = Object::count_by_owner(&conn, owner)?;

        if let Some(id) = params.id {
            let object = Object::select_by_id_owner(&conn, id, owner)?
                .ok_or(Response::not_found(format!("object {} does not exist", id)))?;

            let object_item = ObjectItem {
                id: object.id(),
                owner: object.owner,
                symbol: object.symbol,
                alias: object.alias,
                remark: object.remark,
                created_at: object.created_at,
                updated_at: object.updated_at,
            };

            return Ok(Response::ok(ResponseBody {
                objects: vec![object_item],
                total,
            }));
        }

        let (limit, offset) = paginate(params.page.unwrap_or(1), params.page_size.unwrap_or(256));

        let objects = Object::select_by_owner(&conn, owner, limit, offset)?;

        let objects = objects
            .into_iter()
            .map(|obj| ObjectItem {
                id: obj.id(),
                owner: obj.owner,
                symbol: obj.symbol,
                alias: obj.alias,
                remark: obj.remark,
                created_at: obj.created_at,
                updated_at: obj.updated_at,
            })
            .collect();

        Ok(Response::ok(ResponseBody { objects, total }))
    }
}

mod post {
    pub const PATH: &str = "/finance/objects";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::object::Object;

    #[derive(Debug, Clone, Validate, Serialize, Deserialize)]
    pub struct RequestBody {
        #[validate(length(min = 1, max = 1024))]
        pub symbol: String,
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
        let id = Object::insert(&conn, owner, payload.symbol, payload.alias, payload.remark)?;

        let created_at = Utc::now();

        Ok(Response::ok(ResponseBody { id, created_at }))
    }
}

mod put {
    pub const PATH: &str = "/finance/objects/:id";

    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::object::Object;

    #[derive(Debug, Clone, Validate, Serialize, Deserialize)]
    pub struct RequestBody {
        #[validate(length(min = 2, max = 1024))]
        pub symbol: Option<String>,
        #[validate(length(min = 2, max = 4096))]
        pub alias: Option<String>,
        #[validate(length(min = 2, max = 4096))]
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

        let object = Object::select_by_id_owner(&conn, id, owner)?
            .ok_or(Response::not_found(format!("object {} does not exist", id)))?;

        let symbol = payload.symbol.unwrap_or(object.symbol);
        let alias = payload.alias.or(object.alias);
        let remark = payload.remark.or(object.remark);

        Object::update_by_id_owner(&conn, id, owner, symbol, alias, remark)?;

        Ok(Response::ok(ResponseBody { id }))
    }
}

mod delete {
    pub const PATH: &str = "/finance/objects/:id";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::object::Object;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ObjectItem {
        pub id: i64,
        pub owner: i64,
        pub symbol: String,
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
    pub async fn handler(claim: Claim, Path(id): Path<i64>) -> ResponseResult<ObjectItem> {
        let owner = claim.subject();
        let conn = connection()?;

        let object = Object::select_by_id_owner(&conn, id, owner)?
            .ok_or(Response::not_found(format!("object {} does not exist", id)))?;

        Object::delete_by_id_owner(&conn, id, owner)?;

        let object_item = ObjectItem {
            id: object.id(),
            owner: object.owner,
            symbol: object.symbol,
            alias: object.alias,
            remark: object.remark,
            created_at: object.created_at,
            updated_at: object.updated_at,
        };

        Ok(Response::ok(object_item))
    }
}
