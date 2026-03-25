use clap::Parser;
use dpw4::{verification::HeaderVerifier, HEADER_SIZE};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Header Audit Tool
///
/// Rapidly verifies Fletcher-32 checksums across a binary artifact.
#[derive(Parser)]
struct Cli {
    /// Path to the binary file
    file: PathBuf,

    /// Smoke test mode: check only every Nth header
    #[arg(short, long, default_value = "1")]
    smoke: usize,

    /// Frame size in bytes (header + payload)
    /// Precision-DPW generates frames with fixed metadata.
    /// Default matches standard forensic artifacts (64 bytes).
    ///
    /// # Important
    /// This value must be ≥ 64 to account for the mandatory header.
    #[arg(short, long, default_value = "64")]
    frame_size: usize,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.frame_size < HEADER_SIZE {
        eprintln!(
            "ERROR: frame_size must be at least {} bytes (header size).",
            HEADER_SIZE
        );
        std::process::exit(1);
    }

    let mut file = File::open(&cli.file)?;
    let file_size = file.metadata()?.len();

    // 1.7 Contract: Read full frame_size
    let mut buffer = vec![0u8; cli.frame_size];
    let mut offset: u64 = 0;
    let mut frame_count = 0;
    let mut error_count = 0;

    println!("Auditing {} ({} bytes)...", cli.file.display(), file_size);

    while offset + cli.frame_size as u64 <= file_size {
        file.seek(SeekFrom::Start(offset))?;
        if let Err(e) = file.read_exact(&mut buffer) {
            eprintln!("Read error at offset {}: {}", offset, e);
            break;
        }

        // 1.7 Contract: Call verify_frame_exact(&buffer[..64])
        // Explicit slicing is required.
        if let Err(e) = HeaderVerifier::verify_frame_exact(&buffer[..HEADER_SIZE]) {
            eprintln!(
                "FAILURE at frame {} (offset {}): {}",
                frame_count, offset, e
            );
            error_count += 1;
        }

        frame_count += 1;
        offset += (cli.frame_size * cli.smoke) as u64;
    }

    if error_count == 0 {
        println!("✓ Audit PASSED. Checked {} headers.", frame_count);
        std::process::exit(0);
    } else {
        println!(
            "❌ Audit FAILED. Found {} integrity violations.",
            error_count
        );
        std::process::exit(2);
    }
}
