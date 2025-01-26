use crate::api::http::state::StateInner;

pub fn router(state: std::sync::Arc<StateInner>) -> axum::Router {
    use axum::routing::{delete, get, post, put};

    let router = axum::Router::new()
        .route(get::PATH, get(get::handler))
        .route(post::PATH, post(post::handler))
        .route(put::PATH, put(put::handler))
        .route(delete::PATH, delete(delete::handler))
        .with_state(state);

    router
}

mod get {
    pub const PATH: &str = "/finance/trades/:trade_id/transactions";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::paginate;
    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::transaction::Transaction;
    use crate::model::finance::trade::Trade;
    use crate::model::finance::Quantity;

    #[derive(Debug, Validate, Deserialize)]
    pub struct Params {
        pub id: Option<i64>,
        #[validate(range(min = 1))]
        pub page: Option<usize>,
        #[validate(range(min = 1, max = 1024))]
        pub page_size: Option<usize>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransactionItem {
        pub id: i64,
        pub trade_id: i64,
        pub quantity: Quantity,
        pub is_base_to_quote: bool,
        pub alias: Option<String>,
        pub remark: Option<String>,
        pub occurrence_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub transactions: Vec<TransactionItem>,
        pub total: usize,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Path(trade_id): Path<i64>,
        Query(params): Query<Params>,
    ) -> ResponseResult<ResponseBody> {
        params.validate()?;

        let owner = claim.subject();
        let conn = connection()?;

        let trade = Trade::select_by_id_owner(&conn, trade_id, owner)?.ok_or(
            Response::not_found(format!("trade {} does not exist", trade_id)),
        )?;

        let total = Transaction::count_by_trade_id(&conn, trade.id())?;

        if let Some(id) = params.id {
            let transaction = Transaction::select_by_id_trade_id(&conn, id, trade_id)?.ok_or(
                Response::not_found(format!("transaction {} does not exist", id)),
            )?;

            let transaction_item = TransactionItem {
                id: transaction.id(),
                trade_id: transaction.trade_id,
                quantity: transaction.quantity,
                is_base_to_quote: transaction.is_base_to_quote,
                alias: transaction.alias,
                remark: transaction.remark,
                occurrence_at: transaction.occurrence_at,
                created_at: transaction.created_at,
                updated_at: transaction.updated_at,
            };

            return Ok(Response::ok(ResponseBody {
                transactions: vec![transaction_item],
                total,
            }));
        }

        let (limit, offset) = paginate(params.page.unwrap_or(1), params.page_size.unwrap_or(256));

        let transactions = Transaction::select_by_trade_id(&conn, trade_id, limit, offset)?;

        let transactions = transactions
            .into_iter()
            .map(|tx| TransactionItem {
                id: tx.id(),
                trade_id: tx.trade_id,
                quantity: tx.quantity,
                is_base_to_quote: tx.is_base_to_quote,
                alias: tx.alias,
                remark: tx.remark,
                occurrence_at: tx.occurrence_at,
                created_at: tx.created_at,
                updated_at: tx.updated_at,
            })
            .collect();

        Ok(Response::ok(ResponseBody {
            transactions,
            total,
        }))
    }
}

mod post {
    pub const PATH: &str = "/finance/trades/:trade_id/transactions";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::transaction::Transaction;
    use crate::model::finance::trade::Trade;
    use crate::model::finance::Quantity;

    #[derive(Debug, Clone, Validate, Serialize, Deserialize)]
    pub struct RequestBody {
        pub quantity: Quantity,
        pub is_base_to_quote: bool,
        #[validate(length(min = 1, max = 4096))]
        pub alias: Option<String>,
        #[validate(length(min = 1, max = 4096))]
        pub remark: Option<String>,
        pub occurrence_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub id: i64,
        pub created_at: DateTime<Utc>,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Path(trade_id): Path<i64>,
        Json(payload): Json<RequestBody>,
    ) -> ResponseResult<ResponseBody> {
        payload.validate()?;

        let owner = claim.subject();
        let conn = connection()?;

        let trade = Trade::select_by_id_owner(&conn, trade_id, owner)?.ok_or(
            Response::not_found(format!("trade {} does not exist", trade_id)),
        )?;

        let id = Transaction::insert(
            &conn,
            trade.id(),
            payload.quantity,
            payload.is_base_to_quote,
            payload.alias,
            payload.remark,
            payload.occurrence_at,
        )?;

        let created_at = Utc::now();

        Ok(Response::ok(ResponseBody { id, created_at }))
    }
}

mod put {
    pub const PATH: &str = "/finance/trades/:trade_id/transactions/:id";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::transaction::Transaction;
    use crate::model::finance::trade::Trade;
    use crate::model::finance::Quantity;

    #[derive(Debug, Clone, Validate, Serialize, Deserialize)]
    pub struct RequestBody {
        pub quantity: Option<Quantity>,
        pub is_base_to_quote: Option<bool>,
        #[validate(length(min = 1, max = 4096))]
        pub alias: Option<String>,
        #[validate(length(min = 1, max = 4096))]
        pub remark: Option<String>,
        pub occurrence_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub id: i64,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Path((trade_id, id)): Path<(i64, i64)>,
        Json(payload): Json<RequestBody>,
    ) -> ResponseResult<ResponseBody> {
        payload.validate()?;

        let owner = claim.subject();
        let conn = connection()?;

        let trade = Trade::select_by_id_owner(&conn, trade_id, owner)?.ok_or(
            Response::not_found(format!("trade {} does not exist", trade_id)),
        )?;

        let transaction = Transaction::select_by_id_trade_id(&conn, id, trade.id())?.ok_or(
            Response::not_found(format!("transaction {} does not exist", id)),
        )?;

        let quantity = payload.quantity.unwrap_or(transaction.quantity);
        let is_base_to_quote = payload
            .is_base_to_quote
            .unwrap_or(transaction.is_base_to_quote);
        let alias = payload.alias.or(transaction.alias);
        let remark = payload.remark.or(transaction.remark);
        let occurrence_at = payload.occurrence_at.unwrap_or(transaction.occurrence_at);

        Transaction::update_by_id_trade_id(
            &conn,
            id,
            trade.id(),
            quantity,
            is_base_to_quote,
            occurrence_at,
            alias,
            remark,
        )?;

        Ok(Response::ok(ResponseBody { id }))
    }
}

mod delete {
    pub const PATH: &str = "/finance/trades/:trade_id/transactions/:id";

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::api::http::prelude::*;
    use crate::model::database::prelude::*;
    use crate::model::finance::trade::transaction::Transaction;
    use crate::model::finance::trade::Trade;
    use crate::model::finance::Quantity;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransactionItem {
        pub id: i64,
        pub trade_id: i64,
        pub quantity: Quantity,
        pub is_base_to_quote: bool,
        pub alias: Option<String>,
        pub remark: Option<String>,
        pub occurrence_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[tracing::instrument()]
    pub async fn handler(
        claim: Claim,
        Path((trade_id, id)): Path<(i64, i64)>,
    ) -> ResponseResult<TransactionItem> {
        let owner = claim.subject();
        let conn = connection()?;

        let trade = Trade::select_by_id_owner(&conn, trade_id, owner)?.ok_or(
            Response::not_found(format!("trade {} does not exist", trade_id)),
        )?;

        let transaction = Transaction::select_by_id_trade_id(&conn, id, trade.id())?.ok_or(
            Response::not_found(format!("transaction {} does not exist", id)),
        )?;

        Transaction::delete_by_id_trade_id(&conn, id, trade.id())?;

        let transaction_item = TransactionItem {
            id: transaction.id(),
            trade_id: transaction.trade_id,
            quantity: transaction.quantity,
            is_base_to_quote: transaction.is_base_to_quote,
            alias: transaction.alias,
            remark: transaction.remark,
            occurrence_at: transaction.occurrence_at,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        };

        Ok(Response::ok(transaction_item))
    }
}
