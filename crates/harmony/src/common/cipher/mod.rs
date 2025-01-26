mod chacha20_poly1305;

use std::error::Error;

pub use chacha20_poly1305::ChaCha20Poly1305;

pub trait Cryptographer: Send + Sync {
    fn encrypt(&self, message: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>>;
    fn decrypt(&self, message: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>>;
}
