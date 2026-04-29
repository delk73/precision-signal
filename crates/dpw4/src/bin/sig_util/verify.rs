use super::VerifyError;
use dpw4::{
    verification::{HeaderVerifier, VerificationError},
    BIT_DEPTH_32, HEADER_SIZE,
};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub(crate) fn run_verify(file_path: PathBuf) -> Result<(), VerifyError> {
    let mut file = File::open(&file_path)?;
    let file_size = file.metadata()?.len();

    if file_size < HEADER_SIZE as u64 {
        return Err(VerifyError::Parse("❌ FAIL: File too small".to_string()));
    }

    let mut header_bytes = [0u8; HEADER_SIZE];
    file.read_exact(&mut header_bytes)?;

    match HeaderVerifier::verify_frame_exact(&header_bytes) {
        Ok(_) => {}
        Err(e) => match e {
            VerificationError::ChecksumMismatch | VerificationError::ReservedBytesNotEmpty => {
                return Err(VerifyError::Integrity(format!("❌ FAIL: {}", e)));
            }
            _ => return Err(VerifyError::Parse(format!("❌ FAIL: {}", e))),
        },
    }

    let rate = u32::from_le_bytes(
        header_bytes[16..20]
            .try_into()
            .map_err(|_| VerifyError::Parse("Invalid rate in header".to_string()))?,
    );
    let depth = u32::from_le_bytes(
        header_bytes[20..24]
            .try_into()
            .map_err(|_| VerifyError::Parse("Invalid depth in header".to_string()))?,
    );

    if depth != BIT_DEPTH_32 {
        return Err(VerifyError::Parse(format!(
            "❌ FAIL: Invalid Depth ({})",
            depth
        )));
    }

    let payload = file_size - HEADER_SIZE as u64;
    if !payload.is_multiple_of(4) {
        return Err(VerifyError::Parse(format!(
            "❌ FAIL: Misaligned Payload ({} bytes)",
            payload
        )));
    }

    let duration = (payload / 4) as f64 / rate as f64;
    eprintln!("✅ VERIFIED: DP32 Reference File");
    eprintln!("   Duration: {:.4} sec", duration);
    Ok(())
}
