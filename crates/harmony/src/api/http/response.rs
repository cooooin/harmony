use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

pub type ResponseResult<T> = Result<Response<T>, Response<()>>;

#[derive(Serialize)]
pub struct Response<T>
where
    T: Serialize,
{
    pub(crate) ok: bool,
    pub(crate) code: u16,
    pub(crate) data: Option<T>,
    pub(crate) message: Option<String>,
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn new() -> Self {
        Self {
            ok: true,
            code: 200,
            data: None,
            message: None,
        }
    }

    pub fn ok(data: T) -> Self {
        let mut response = Self::new();
        response.data = Some(data);

        response
    }

    // pub fn forbidden(message: String) -> Self {
    //     let mut response = Self::new();
    //     response.code = 403;
    //     response.message = Some(message);

    //     response
    // }

    pub fn not_found(message: String) -> Self {
        let mut response = Self::new();
        response.ok = false;
        response.code = 404;
        response.message = Some(message);

        response
    }

    pub fn bad_request(message: String) -> Self {
        let mut response = Self::new();
        response.ok = false;
        response.code = 400;
        response.message = Some(message);

        response
    }

    // pub fn internal_error(message: String) -> Self {
    //     let mut response = Self::new();
    //     response.ok = false;
    //     response.code = 500;
    //     response.message = Some(message);

    //     response
    // }
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let code = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = Json(json!(self));

        (code, body).into_response()
    }
}

mod from_general_error {
    use std::error::Error;

    use super::{Response, Serialize};

    impl<T> From<Box<dyn Error>> for Response<T>
    where
        T: Serialize,
    {
        fn from(value: Box<dyn Error>) -> Self {
            Self::bad_request(value.to_string())
        }
    }
}

mod from_database_error {
    use rusqlite::Error;

    use super::{Response, Serialize};

    impl<T> From<Error> for Response<T>
    where
        T: Serialize,
    {
        fn from(value: Error) -> Self {
            Self::bad_request(value.to_string())
        }
    }
}

mod from_vaild_error {
    use validator::ValidationErrors;

    use super::{Response, Serialize};

    impl<T> From<ValidationErrors> for Response<T>
    where
        T: Serialize,
    {
        fn from(value: ValidationErrors) -> Self {
            Self::bad_request(value.to_string())
        }
    }
}
