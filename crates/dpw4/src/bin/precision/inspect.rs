use dpw4::{verification::HeaderVerifier, HEADER_SIZE};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub(crate) fn run_inspect(file_path: Option<PathBuf>) -> io::Result<()> {
    let mut reader: Box<dyn Read> = match file_path {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };

    let mut header_bytes = [0u8; HEADER_SIZE];
    reader.read_exact(&mut header_bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("Failed to read DP32 header: {}", e),
        )
    })?;

    if let Err(e) = HeaderVerifier::verify_frame_exact(&header_bytes) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("❌ Invalid Header: {}", e),
        ));
    }

    let version = u32::from_le_bytes(
        header_bytes[4..8]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid version field"))?,
    );
    let sequence = u64::from_le_bytes(
        header_bytes[8..16]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid sequence field"))?,
    );
    let rate = u32::from_le_bytes(
        header_bytes[16..20]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid rate field"))?,
    );
    let depth = u32::from_le_bytes(
        header_bytes[20..24]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid depth field"))?,
    );

    eprintln!("=== DP32 Inspector ===");
    eprintln!("Status:      VALID HEADER");
    eprintln!("Protocol v:  {}", version);
    eprintln!("Sample Rate: {} Hz", rate);
    eprintln!("Bit Depth:   {}-bit (S32LE)", depth);
    eprintln!("Sequence:    {}", sequence);
    Ok(())
}
