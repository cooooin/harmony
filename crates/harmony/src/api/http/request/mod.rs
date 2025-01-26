mod access;

use axum::async_trait;
use axum::extract::Path as AxumPath;
use axum::extract::Query as AxumQuery;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::Json as AxumJson;
use serde::de::DeserializeOwned;

use crate::api::http::response::Response;

pub mod headers {
    use super::*;

    pub use super::Path;
    pub use super::Query;
    pub use access::claim::Claim;
}

pub mod body {
    pub use super::Json;
}

// ===== Query =====
#[derive(Debug, Clone)]
pub struct Query<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response<()>;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match AxumQuery::try_from_uri(&parts.uri) {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let response = Response::bad_request(rejection.body_text());

                Err(response)
            }
        }
    }
}

// ===== Path =====
#[derive(Debug, Clone)]
pub struct Path<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response<()>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AxumPath::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let response = Response::bad_request(rejection.body_text());

                Err(response)
            }
        }
    }
}

// ===== JSON =====
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Json<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response<()>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match AxumJson::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let response = Response::bad_request(rejection.body_text());

                Err(response)
            }
        }
    }
}

// ===== Multipart =====
// use axum::extract::Multipart as AxumMultipart;
// #[derive(Debug)]
// pub struct Multipart(pub AxumMultipart);

// #[async_trait]
// impl<S> FromRequest<S> for Multipart
// where
//     S: Send + Sync,
// {
//     type Rejection = Response<()>;

//     async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
//         match AxumMultipart::from_request(req, state).await {
//             Ok(value) => Ok(Self(value)),
//             Err(rejection) => {
//                 let response = Response::bad_request(rejection.body_text());

//                 Err(response)
//             }
//         }
//     }
// }
