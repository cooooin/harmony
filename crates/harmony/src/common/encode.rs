use std::error::Error;

use base64::Engine;

pub fn base64_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD as Base64;

    Base64.encode(data)
}

pub fn base64_decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    use base64::engine::general_purpose::STANDARD as Base64;

    match Base64.decode(data) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.to_string().into()),
    }
}
