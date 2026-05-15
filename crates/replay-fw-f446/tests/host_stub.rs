#[path = "../src/artifact_metadata.rs"]
mod artifact_metadata;
#[path = "../src/signal_model.rs"]
mod signal_model;

use sha2::{Digest, Sha256};
use signal_model::{
    advance_state_for_model, persistent_divergence_state, sample_for_model, SignalModel,
    SELECTED_SIGNAL_MODEL, SIGNAL_INITIAL_STATE,
};

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

#[test]
fn selected_signal_model_is_phase8_when_default_or_explicit_phase8() {
    assert_eq!(SELECTED_SIGNAL_MODEL, SignalModel::Phase8);
    assert_eq!(SIGNAL_INITIAL_STATE, 0);
}

#[test]
fn signal_model_samples_match_contract() {
    assert_eq!(sample_for_model(SignalModel::Phase8, 0, 0), 0);
    assert_eq!(
        sample_for_model(SignalModel::Phase8, 255, 0xFF00_0000),
        0xFF
    );
    assert_eq!(sample_for_model(SignalModel::Phase8, 256, 0), 0);

    assert_eq!(sample_for_model(SignalModel::Burst8, 0, 0), 0);
    assert_eq!(sample_for_model(SignalModel::Burst8, 47, 0), 0);
    assert_eq!(sample_for_model(SignalModel::Burst8, 48, 0), 1);
    assert_eq!(sample_for_model(SignalModel::Burst8, 63, 0), 16);
    assert_eq!(sample_for_model(SignalModel::Burst8, 64, 0), 0);

    let mut state = 0xA5;
    for expected in [0xA5, 0xEA, 0x75, 0x82, 0x41] {
        assert_eq!(
            sample_for_model(SignalModel::SeededLfsr8, 0, state),
            expected
        );
        state = advance_state_for_model(SignalModel::SeededLfsr8, state);
    }
}

#[test]
fn persistent_divergence_state_is_explicit_per_model() {
    assert_eq!(
        persistent_divergence_state(SignalModel::Phase8, 0),
        Some(advance_state_for_model(SignalModel::Phase8, 0))
    );
    assert_eq!(
        persistent_divergence_state(SignalModel::SeededLfsr8, 0xA5),
        Some(advance_state_for_model(SignalModel::SeededLfsr8, 0xA5))
    );
    assert_eq!(persistent_divergence_state(SignalModel::Burst8, 0), None);
}
