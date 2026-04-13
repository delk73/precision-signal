# Precision CLI Examples
**Note:** These examples assume the `precision` binary is in your path or run via `cargo run --`.

## 1. Inspect a DP32 Header
Inspect the metadata of a reference signal file to verify sample rate and alignment.

```bash
# Generate a 1-second file first
cargo run -- generate --seconds 1 > ref.dp32

# Inspect it
cargo run -- inspect --file ref.dp32
```

**Expected Output:**
```text
=== DP32 Inspector ===
Status:      VALID HEADER
Protocol v:  1
Sample Rate: 48000 Hz
Bit Depth:   32-bit (S32LE)
Sequence:    0
```

## 2. Verify a DP32 File
Perform a sanity check on file size, magic, and payload alignment.

```bash
cargo run -- verify --file ref.dp32
```

**Expected Output:**
```text
✅ VERIFIED: DP32 Reference File
   Duration: 1.0000 sec
```

## 3. Emit WAV for Monitoring
Generate a 2-second sine wave at 440Hz and wrap it in a WAV container for playback or inspection.

```bash
cargo run -- generate --shape saw --freq 440 --seconds 2 --container-wav > alert.wav
```

## 4. Perform a Manual Forensic Audit
Calculate the SHA-256 hash of the payload (the raw signal) of a DP32 file.

```bash
# Generate the signal
cargo run -- generate --shape saw --freq 440 --rate 44100 --seconds 1 > test.dp32

# Hash the payload (skip the 64-byte header)
tail -c +65 test.dp32 | sha256sum
```

**Note:** The resulting hash must match the reference values in `forensic_audit.rs` for the specific parameters (shape, frequency, rate).

## 5. Run Canonical Determinism Validation
Run the normative release gate with stepwise checks.

```bash
cargo run --release -p dpw4 --features cli --bin sig-util -- validate --mode quick
```

## 6. Generate Forensic Artifacts via CLI
Emit the advisory forensic traces into a chosen directory.

```bash
cargo run -p dpw4 --features cli --bin precision -- artifacts --out docs/verification
```