#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignalModel {
    Phase8,
    Burst8,
    SeededLfsr8,
}

#[cfg(any(
    all(feature = "signal-model-phase8", feature = "signal-model-burst8"),
    all(feature = "signal-model-phase8", feature = "signal-model-seeded-lfsr8"),
    all(feature = "signal-model-burst8", feature = "signal-model-seeded-lfsr8")
))]
compile_error!("select at most one replay signal model feature");

pub const PHASE8_STEP: u32 = 0x0100_0000;
pub const BURST8_PERIOD: u32 = 64;
pub const BURST8_IDLE: u32 = 48;
pub const SEEDED_LFSR8_SEED: u8 = 0xA5;
pub const SEEDED_LFSR8_TAP_MASK: u8 = 0xB8;

#[cfg(all(
    feature = "signal-model-phase8",
    not(feature = "signal-model-burst8"),
    not(feature = "signal-model-seeded-lfsr8")
))]
pub const SELECTED_SIGNAL_MODEL: SignalModel = SignalModel::Phase8;

#[cfg(all(
    not(feature = "signal-model-phase8"),
    feature = "signal-model-burst8",
    not(feature = "signal-model-seeded-lfsr8")
))]
pub const SELECTED_SIGNAL_MODEL: SignalModel = SignalModel::Burst8;

#[cfg(all(
    not(feature = "signal-model-phase8"),
    not(feature = "signal-model-burst8"),
    feature = "signal-model-seeded-lfsr8"
))]
pub const SELECTED_SIGNAL_MODEL: SignalModel = SignalModel::SeededLfsr8;

#[cfg(all(
    not(feature = "signal-model-phase8"),
    not(feature = "signal-model-burst8"),
    not(feature = "signal-model-seeded-lfsr8")
))]
pub const SELECTED_SIGNAL_MODEL: SignalModel = SignalModel::Phase8;

pub const SIGNAL_INITIAL_STATE: u32 = match SELECTED_SIGNAL_MODEL {
    SignalModel::Phase8 | SignalModel::Burst8 => 0,
    SignalModel::SeededLfsr8 => SEEDED_LFSR8_SEED as u32,
};

pub fn sample_for_model(model: SignalModel, frame_idx: u32, state: u32) -> i32 {
    match model {
        SignalModel::Phase8 => (state >> 24) as i32,
        SignalModel::Burst8 => burst8_sample(frame_idx),
        SignalModel::SeededLfsr8 => (state & 0xFF) as i32,
    }
}

pub fn advance_state_for_model(model: SignalModel, state: u32) -> u32 {
    match model {
        SignalModel::Phase8 => state.wrapping_add(PHASE8_STEP),
        SignalModel::Burst8 => state,
        SignalModel::SeededLfsr8 => u32::from(advance_lfsr8(state as u8)),
    }
}

fn burst8_sample(frame_idx: u32) -> i32 {
    let cycle_pos = frame_idx % BURST8_PERIOD;
    if cycle_pos < BURST8_IDLE {
        return 0;
    }

    let burst_pos = cycle_pos - BURST8_IDLE;
    let burst_epoch = frame_idx / BURST8_PERIOD;
    ((burst_epoch + burst_pos + 1) & 0xFF) as i32
}

fn advance_lfsr8(state: u8) -> u8 {
    let shifted = state >> 1;
    if (state & 1) == 0 {
        shifted
    } else {
        shifted ^ SEEDED_LFSR8_TAP_MASK
    }
}
