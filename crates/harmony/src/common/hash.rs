use ring::digest::{Context, Digest, SHA256};
use std::fmt::Write;

/// Computes the SHA-256 digest of the input data.
pub fn sha256_digest(input: &[u8]) -> Digest {
    let mut context = Context::new(&SHA256);
    context.update(input);
    context.finish()
}

/// Converts a SHA-256 digest to a hexadecimal string.
/// Returns `None` if formatting fails.
pub fn digest_to_hex(digest: &Digest) -> Option<String> {
    let mut hex_string = String::new();
    for byte in digest.as_ref() {
        if write!(&mut hex_string, "{:02x}", byte).is_err() {
            return None; // Return None if formatting fails
        }
    }
    Some(hex_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_digest() {
        let data = b"hello, world";
        let digest = sha256_digest(data);

        // Verify the length of the digest (SHA-256 digest should be 32 bytes)
        assert_eq!(digest.as_ref().len(), 32);

        // Verify the value of the digest
        let expected_digest = [
            0x09, 0xca, 0x7e, 0x4e, 0xaa, 0x6e, 0x8a, 0xe9, 0xc7, 0xd2, 0x61, 0x16, 0x71, 0x29,
            0x18, 0x48, 0x83, 0x64, 0x4d, 0x07, 0xdf, 0xba, 0x7c, 0xbf, 0xbc, 0x4c, 0x8a, 0x2e,
            0x08, 0x36, 0x0d, 0x5b,
        ];
        assert_eq!(digest.as_ref(), expected_digest);
    }

    #[test]
    fn test_digest_to_hex() {
        let data = b"hello, world";
        let digest = sha256_digest(data);

        // Verify the hexadecimal string conversion
        let hex_string = digest_to_hex(&digest).unwrap();
        assert_eq!(
            hex_string,
            "09ca7e4eaa6e8ae9c7d261167129184883644d07dfba7cbfbc4c8a2e08360d5b"
        );

        // Test empty input
        let empty_digest = sha256_digest(b"");
        let empty_hex_string = digest_to_hex(&empty_digest).unwrap();
        assert_eq!(
            empty_hex_string,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
