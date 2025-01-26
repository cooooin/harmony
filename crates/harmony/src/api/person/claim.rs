pub(super) mod post {
    pub const PATH: &str = "/person/claim";

    use serde::{Deserialize, Serialize};

    use crate::api::http::prelude::*;
    use crate::api::person::validate_value;
    use crate::model::database::prelude::*;
    use crate::model::person::Person;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RequestBody {
        pub nickname: String,
        pub password: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        pub claim: String,
        pub expire: u128,
    }

    #[tracing::instrument()]
    pub async fn handler(Json(payload): Json<RequestBody>) -> ResponseResult<ResponseBody> {
        let nickname = validate_value::nickname(payload.nickname)?;
        let password = validate_value::password(payload.password)?;
        let connection = connection()?;

        let person = Person::select_one_by_nickname_password(&connection, &nickname, &password)?
            .ok_or(Response::bad_request(
                "incorrect nickname or password".into(),
            ))?;

        let claim = Claim::new(person.id());
        let expire = claim.expire();

        Ok(Response::ok(ResponseBody {
            claim: claim.issue()?,
            expire,
        }))
    }
}
