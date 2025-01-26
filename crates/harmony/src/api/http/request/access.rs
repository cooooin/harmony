use super::*;

// ===== Claim =====
pub mod claim {
    use std::error::Error;

    use serde::{Deserialize, Serialize};

    use crate::common::cipher::Cryptographer;

    use super::*;

    const HEADER: &str = "X-Access-Claim";

    #[async_trait]
    impl<S> FromRequestParts<S> for Claim
    where
        S: Send + Sync,
    {
        type Rejection = Response<()>;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let ciphertext = parts
                .headers
                .get(HEADER)
                .ok_or(Response::bad_request("not provide claim".into()))?
                .as_bytes();

            match Claim::verify(ciphertext) {
                Ok(claim) => {
                    if !claim.is_expire() {
                        Ok(claim)
                    } else {
                        return Err(Response::bad_request("expired claim".into()));
                    }
                }
                Err(_e) => Err(Response::bad_request("invalid claim".into())),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Claim {
        sub: i64,
        iat: u128,
        exp: u128,
    }

    impl Claim {
        pub fn new(sub: i64) -> Self {
            use crate::time::timestamp;

            let iat = timestamp().as_millis();
            let exp = iat + 7 * 24 * 60 * 60 * 1000;

            Self { sub, iat, exp }
        }

        pub fn subject(&self) -> i64 {
            self.sub
        }

        pub fn expire(&self) -> u128 {
            self.exp
        }

        pub fn is_expire(&self) -> bool {
            use crate::time::timestamp;

            self.exp < timestamp().as_millis()
        }

        pub fn issue(&self) -> Result<String, Box<dyn Error>> {
            use crate::common::encode::base64_encode;
            use crate::consts::claim_encrypt::ENCRYPTER;

            let message = serde_json::to_vec(&self)?;
            let ciphertext = ENCRYPTER.encrypt(message)?;
            let result = base64_encode(&ciphertext);

            Ok(result)
        }

        pub fn verify(value: &[u8]) -> Result<Self, Box<dyn Error>> {
            use crate::common::encode::base64_decode;
            use crate::consts::claim_encrypt::ENCRYPTER;

            let message = base64_decode(value)?;
            let ciphertext = ENCRYPTER.decrypt(message)?;
            let result = serde_json::from_slice(&ciphertext)?;

            Ok(result)
        }
    }
}
