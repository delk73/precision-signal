#![allow(clippy::module_name_repetitions)]

/// Artifact stream magic bytes for Sprint 1 wire format.
pub const MAGIC: [u8; 4] = *b"RPL0";

/// Artifact stream version for Sprint 1 wire format.
pub const VERSION0: u32 = 0;
pub const VERSION1: u16 = 1;

/// Fixed v0 wire constants. The wire contract boundary is encoder output.
pub const HEADER_SIZE: usize = 16;
pub const FRAME_SIZE: usize = 16;
pub const FRAME_COUNT: usize = 10_000;
pub const ARTIFACT0_SIZE: usize = 160_016;
pub const V1_MIN_HEADER_SIZE: usize = 0x98;
pub const HEADER1_SIZE: usize = V1_MIN_HEADER_SIZE;

// Fixed v1 field offsets.
pub const V1_OFF_VERSION: usize = 0x04;
pub const V1_OFF_HEADER_LEN: usize = 0x06;
pub const V1_OFF_FRAME_COUNT: usize = 0x08;
pub const V1_OFF_FRAME_SIZE: usize = 0x0C;
pub const V1_OFF_FLAGS: usize = 0x0E;
pub const V1_OFF_SCHEMA_LEN: usize = 0x10;
pub const V1_OFF_SCHEMA_HASH: usize = 0x14;
pub const V1_OFF_BUILD_HASH: usize = 0x34;
pub const V1_OFF_CONFIG_HASH: usize = 0x54;
pub const V1_OFF_BOARD_ID: usize = 0x74;
pub const V1_OFF_CLOCK_PROFILE: usize = 0x84;
pub const V1_OFF_CAPTURE_BOUNDARY: usize = 0x94;
pub const V1_OFF_RESERVED: usize = 0x96;

pub const SCHEMA_HASH_SIZE: usize = 32;
pub const BUILD_HASH_SIZE: usize = 32;
pub const CONFIG_HASH_SIZE: usize = 32;
pub const BOARD_ID_SIZE: usize = 16;
pub const CLOCK_PROFILE_SIZE: usize = 16;

// Compile-time wire-size locks for v0.
const _: [(); HEADER_SIZE] = [(); 16];
const _: [(); FRAME_SIZE] = [(); 16];
const _: [(); ARTIFACT0_SIZE] = [(); HEADER_SIZE + FRAME_COUNT * FRAME_SIZE];

/// Canonical little-endian header for replay artifact v0.
///
/// `repr(C)` is retained as implementation layout discipline.
/// The normative wire contract is defined by explicit encoder outputs.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Header0 {
    pub magic: [u8; 4],
    pub version: u32,
    pub frame_count: u32,
    pub reserved: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header1 {
    pub magic: [u8; 4],
    pub version: u16,
    pub header_len: u16,
    pub frame_count: u32,
    pub frame_size: u16,
    pub flags: u16,
    pub schema_len: u32,
    pub schema_hash: [u8; SCHEMA_HASH_SIZE],
    pub build_hash: [u8; BUILD_HASH_SIZE],
    pub config_hash: [u8; CONFIG_HASH_SIZE],
    pub board_id: [u8; BOARD_ID_SIZE],
    pub clock_profile: [u8; CLOCK_PROFILE_SIZE],
    pub capture_boundary: u16,
    pub reserved: u16,
}

/// Canonical little-endian event frame for replay artifact v0.
///
/// `repr(C)` is retained as implementation layout discipline.
/// The normative wire contract is defined by explicit encoder outputs.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EventFrame0 {
    pub frame_idx: u32,
    pub irq_id: u8,
    pub flags: u8,
    pub rsv: u16,
    pub timer_delta: u32,
    pub input_sample: i32,
}

// Backward-compat aliases used by host/firmware code paths.
pub const HEADER0_SIZE: usize = HEADER_SIZE;
pub const EVENTFRAME0_SIZE: usize = FRAME_SIZE;

#[allow(dead_code)]
pub(crate) fn encode_header0_le(header: &Header0) -> [u8; HEADER_SIZE] {
    let mut out = [0u8; HEADER_SIZE];
    out[0..4].copy_from_slice(&header.magic);
    out[4..8].copy_from_slice(&header.version.to_le_bytes());
    out[8..12].copy_from_slice(&header.frame_count.to_le_bytes());
    out[12..16].copy_from_slice(&header.reserved.to_le_bytes());
    out
}

#[allow(dead_code)]
pub fn encode_header1_le(header: &Header1) -> [u8; HEADER1_SIZE] {
    let mut out = [0u8; HEADER1_SIZE];
    out[0..4].copy_from_slice(&header.magic);
    out[V1_OFF_VERSION..V1_OFF_VERSION + 2].copy_from_slice(&header.version.to_le_bytes());
    out[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2].copy_from_slice(&header.header_len.to_le_bytes());
    out[V1_OFF_FRAME_COUNT..V1_OFF_FRAME_COUNT + 4]
        .copy_from_slice(&header.frame_count.to_le_bytes());
    out[V1_OFF_FRAME_SIZE..V1_OFF_FRAME_SIZE + 2].copy_from_slice(&header.frame_size.to_le_bytes());
    out[V1_OFF_FLAGS..V1_OFF_FLAGS + 2].copy_from_slice(&header.flags.to_le_bytes());
    out[V1_OFF_SCHEMA_LEN..V1_OFF_SCHEMA_LEN + 4].copy_from_slice(&header.schema_len.to_le_bytes());
    out[V1_OFF_SCHEMA_HASH..V1_OFF_SCHEMA_HASH + SCHEMA_HASH_SIZE]
        .copy_from_slice(&header.schema_hash);
    out[V1_OFF_BUILD_HASH..V1_OFF_BUILD_HASH + BUILD_HASH_SIZE].copy_from_slice(&header.build_hash);
    out[V1_OFF_CONFIG_HASH..V1_OFF_CONFIG_HASH + CONFIG_HASH_SIZE]
        .copy_from_slice(&header.config_hash);
    out[V1_OFF_BOARD_ID..V1_OFF_BOARD_ID + BOARD_ID_SIZE].copy_from_slice(&header.board_id);
    out[V1_OFF_CLOCK_PROFILE..V1_OFF_CLOCK_PROFILE + CLOCK_PROFILE_SIZE]
        .copy_from_slice(&header.clock_profile);
    out[V1_OFF_CAPTURE_BOUNDARY..V1_OFF_CAPTURE_BOUNDARY + 2]
        .copy_from_slice(&header.capture_boundary.to_le_bytes());
    out[V1_OFF_RESERVED..V1_OFF_RESERVED + 2].copy_from_slice(&header.reserved.to_le_bytes());
    out
}

pub fn encode_event_frame0_le(frame: &EventFrame0) -> [u8; FRAME_SIZE] {
    let mut out = [0u8; FRAME_SIZE];
    out[0..4].copy_from_slice(&frame.frame_idx.to_le_bytes());
    out[4] = frame.irq_id;
    out[5] = frame.flags;
    out[6..8].copy_from_slice(&frame.rsv.to_le_bytes());
    out[8..12].copy_from_slice(&frame.timer_delta.to_le_bytes());
    out[12..16].copy_from_slice(&frame.input_sample.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_sizes_match_versioned_wire_constants() {
        assert_eq!(encode_header0_le(&Header0::default()).len(), HEADER_SIZE);
        let header1 = Header1 {
            magic: MAGIC,
            version: VERSION1,
            header_len: V1_MIN_HEADER_SIZE as u16,
            frame_count: 0,
            frame_size: FRAME_SIZE as u16,
            flags: 0,
            schema_len: 0,
            schema_hash: [0u8; SCHEMA_HASH_SIZE],
            build_hash: [0u8; BUILD_HASH_SIZE],
            config_hash: [0u8; CONFIG_HASH_SIZE],
            board_id: [0u8; BOARD_ID_SIZE],
            clock_profile: [0u8; CLOCK_PROFILE_SIZE],
            capture_boundary: 0,
            reserved: 0,
        };
        assert_eq!(encode_header1_le(&header1).len(), HEADER1_SIZE);
        assert_eq!(
            encode_event_frame0_le(&EventFrame0::default()).len(),
            FRAME_SIZE
        );
        assert_eq!(HEADER_SIZE + FRAME_COUNT * FRAME_SIZE, ARTIFACT0_SIZE);
    }
    #[test]
    fn encode_header1_places_key_fields_at_v1_offsets() {
        let header = Header1 {
            magic: MAGIC,
            version: VERSION1,
            header_len: 0x0098,
            frame_count: 0x0102_0304,
            frame_size: FRAME_SIZE as u16,
            flags: 0,
            schema_len: 0x0A0B_0C0D,
            schema_hash: [0x11; SCHEMA_HASH_SIZE],
            build_hash: [0x22; BUILD_HASH_SIZE],
            config_hash: [0x33; CONFIG_HASH_SIZE],
            board_id: [0x44; BOARD_ID_SIZE],
            clock_profile: [0x55; CLOCK_PROFILE_SIZE],
            capture_boundary: 0x1234,
            reserved: 0,
        };

        let encoded = encode_header1_le(&header);

        assert_eq!(
            &encoded[V1_OFF_VERSION..V1_OFF_VERSION + 2],
            &VERSION1.to_le_bytes()
        );
        assert_eq!(
            &encoded[V1_OFF_HEADER_LEN..V1_OFF_HEADER_LEN + 2],
            &header.header_len.to_le_bytes()
        );
        assert_eq!(
            &encoded[V1_OFF_FRAME_COUNT..V1_OFF_FRAME_COUNT + 4],
            &header.frame_count.to_le_bytes()
        );
        assert_eq!(
            &encoded[V1_OFF_FRAME_SIZE..V1_OFF_FRAME_SIZE + 2],
            &header.frame_size.to_le_bytes()
        );
        assert_eq!(
            &encoded[V1_OFF_SCHEMA_LEN..V1_OFF_SCHEMA_LEN + 4],
            &header.schema_len.to_le_bytes()
        );
        assert_eq!(
            &encoded[V1_OFF_CAPTURE_BOUNDARY..V1_OFF_CAPTURE_BOUNDARY + 2],
            &header.capture_boundary.to_le_bytes()
        );
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    fn any_header0() -> Header0 {
        Header0 {
            magic: kani::any(),
            version: kani::any(),
            frame_count: kani::any(),
            reserved: kani::any(),
        }
    }

    fn any_event_frame0() -> EventFrame0 {
        EventFrame0 {
            frame_idx: kani::any(),
            irq_id: kani::any(),
            flags: kani::any(),
            rsv: kani::any(),
            timer_delta: kani::any(),
            input_sample: kani::any(),
        }
    }

    #[kani::proof]
    fn proof_v0_wire_size_constants_use_artifact0_size() {
        assert_eq!(encode_header0_le(&Header0::default()).len(), HEADER_SIZE);
        assert_eq!(
            encode_event_frame0_le(&EventFrame0::default()).len(),
            FRAME_SIZE
        );
        assert_eq!(HEADER_SIZE + FRAME_COUNT * FRAME_SIZE, ARTIFACT0_SIZE);
    }

    #[kani::proof]
    fn proof_encode_header0_wire_layout_and_le() {
        let h = any_header0();
        let encoded = encode_header0_le(&h);

        assert_eq!(encoded[0], h.magic[0]);
        assert_eq!(encoded[1], h.magic[1]);
        assert_eq!(encoded[2], h.magic[2]);
        assert_eq!(encoded[3], h.magic[3]);
        assert_eq!(
            u32::from_le_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]),
            h.version
        );
        assert_eq!(
            u32::from_le_bytes([encoded[8], encoded[9], encoded[10], encoded[11]]),
            h.frame_count
        );
        assert_eq!(
            u32::from_le_bytes([encoded[12], encoded[13], encoded[14], encoded[15]]),
            h.reserved
        );

        // Explicit expected byte stream from field encodes only.
        let expected = [
            h.magic[0],
            h.magic[1],
            h.magic[2],
            h.magic[3],
            (h.version & 0xFF) as u8,
            ((h.version >> 8) & 0xFF) as u8,
            ((h.version >> 16) & 0xFF) as u8,
            ((h.version >> 24) & 0xFF) as u8,
            (h.frame_count & 0xFF) as u8,
            ((h.frame_count >> 8) & 0xFF) as u8,
            ((h.frame_count >> 16) & 0xFF) as u8,
            ((h.frame_count >> 24) & 0xFF) as u8,
            (h.reserved & 0xFF) as u8,
            ((h.reserved >> 8) & 0xFF) as u8,
            ((h.reserved >> 16) & 0xFF) as u8,
            ((h.reserved >> 24) & 0xFF) as u8,
        ];
        assert_eq!(encoded, expected);
    }

    #[kani::proof]
    fn proof_encode_event_frame0_wire_layout_and_le() {
        let f = any_event_frame0();
        let encoded = encode_event_frame0_le(&f);

        assert_eq!(
            u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]),
            f.frame_idx
        );
        assert_eq!(encoded[4], f.irq_id);
        assert_eq!(encoded[5], f.flags);
        assert_eq!(u16::from_le_bytes([encoded[6], encoded[7]]), f.rsv);
        assert_eq!(
            u32::from_le_bytes([encoded[8], encoded[9], encoded[10], encoded[11]]),
            f.timer_delta
        );
        assert_eq!(
            i32::from_le_bytes([encoded[12], encoded[13], encoded[14], encoded[15]]),
            f.input_sample
        );

        // Explicit expected byte stream from field encodes only.
        let frame_idx = f.frame_idx;
        let rsv = f.rsv;
        let timer_delta = f.timer_delta;
        let input_sample = f.input_sample as u32;
        let expected = [
            (frame_idx & 0xFF) as u8,
            ((frame_idx >> 8) & 0xFF) as u8,
            ((frame_idx >> 16) & 0xFF) as u8,
            ((frame_idx >> 24) & 0xFF) as u8,
            f.irq_id,
            f.flags,
            (rsv & 0xFF) as u8,
            ((rsv >> 8) & 0xFF) as u8,
            (timer_delta & 0xFF) as u8,
            ((timer_delta >> 8) & 0xFF) as u8,
            ((timer_delta >> 16) & 0xFF) as u8,
            ((timer_delta >> 24) & 0xFF) as u8,
            (input_sample & 0xFF) as u8,
            ((input_sample >> 8) & 0xFF) as u8,
            ((input_sample >> 16) & 0xFF) as u8,
            ((input_sample >> 24) & 0xFF) as u8,
        ];
        assert_eq!(encoded, expected);
    }
}
