use std::error::Error;

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};

use super::Cryptographer;

// ===== ChaCha20-Poly1305 =====
pub struct ChaCha20Poly1305 {
    key: LessSafeKey,
    nonce: [u8; 12],
}

impl ChaCha20Poly1305 {
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Result<Self, Box<dyn Error>> {
        let unbound_key = match UnboundKey::new(&CHACHA20_POLY1305, &key) {
            Ok(value) => value,
            Err(err) => return Err(err.to_string().into()),
        };

        Ok(Self {
            key: LessSafeKey::new(unbound_key),
            nonce,
        })
    }

    fn aad(&self) -> Aad<[u8; 0]> {
        Aad::empty()
    }

    fn nonce(&self) -> Nonce {
        Nonce::assume_unique_for_key(self.nonce)
    }
}

impl Cryptographer for ChaCha20Poly1305 {
    fn encrypt(&self, mut message: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = self.nonce();
        let aad = self.aad();

        if let Err(err) = self.key.seal_in_place_append_tag(nonce, aad, &mut message) {
            return Err(err.to_string().into());
        }

        Ok(message)
    }

    fn decrypt(&self, mut message: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = self.nonce();
        let aad = self.aad();

        let data = match self.key.open_in_place(nonce, aad, &mut message) {
            Ok(v) => v,
            Err(e) => return Err(e.to_string().into()),
        };

        Ok(Vec::from(data))
    }
}

#[cfg(test)]
mod tests {
    use super::{ChaCha20Poly1305, Cryptographer};

    #[test]
    fn test_chacha20_poly1305() {
        let message = String::from("hello");
        let chacha20 = ChaCha20Poly1305::new(
            "12345678901234567890123456789012"
                .as_bytes()
                .try_into()
                .unwrap(),
            "123456789012".as_bytes().try_into().unwrap(),
        )
        .unwrap();

        let encrypt = chacha20.encrypt(message.clone().into()).unwrap();

        assert_eq!(
            encrypt,
            [
                234, 184, 196, 249, 169, 15, 97, 157, 230, 249, 106, 10, 177, 34, 20, 212, 247,
                106, 118, 177, 90
            ]
        );

        let decrypt = chacha20.decrypt(encrypt).unwrap();
        assert_eq!(message, std::string::String::from_utf8(decrypt).unwrap());
    }
}
