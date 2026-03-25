#[path = "../src/artifact_metadata.rs"]
mod artifact_metadata;

use sha2::{Digest, Sha256};

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[test]
fn firmware_digest_constants_match_metadata_inputs() {
    assert_eq!(
        sha256_bytes(artifact_metadata::RPL0_SCHEMA),
        artifact_metadata::SCHEMA_HASH
    );
    assert_eq!(
        sha256_bytes(artifact_metadata::BUILD_HASH_INPUT),
        artifact_metadata::BUILD_HASH
    );
    assert_eq!(
        sha256_bytes(artifact_metadata::CONFIG_HASH_INPUT),
        artifact_metadata::CONFIG_HASH
    );
}
