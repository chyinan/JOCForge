// pattern: Functional Core
//! RFC 2104 HMAC-SHA-256 primitive over caller-supplied key and message bytes.

use sha2::{Digest, Sha256};

const SHA256_BLOCK_BYTES: usize = 64;
const SHA256_DIGEST_BYTES: usize = 32;

/// Computes HMAC-SHA-256 without retrieving, storing, or deriving a key.
///
/// The caller supplies key bytes for this invocation. The F14 production profile separately
/// requires exactly 32 authorized key bytes; this primitive does not provide that key source.
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; SHA256_DIGEST_BYTES] {
    let mut normalized_key = [0_u8; SHA256_BLOCK_BYTES];
    if key.len() > SHA256_BLOCK_BYTES {
        let key_digest = Sha256::digest(key);
        normalized_key[..SHA256_DIGEST_BYTES].copy_from_slice(&key_digest);
    } else {
        normalized_key[..key.len()].copy_from_slice(key);
    }

    let mut inner_pad = [0x36_u8; SHA256_BLOCK_BYTES];
    let mut outer_pad = [0x5c_u8; SHA256_BLOCK_BYTES];
    for index in 0..SHA256_BLOCK_BYTES {
        inner_pad[index] ^= normalized_key[index];
        outer_pad[index] ^= normalized_key[index];
    }

    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}
