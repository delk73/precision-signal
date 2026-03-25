#!/bin/bash
set -e

# Build the binary with the 'cli' feature enabled
cargo build --features cli --bin precision

# Run for 1 second
./target/debug/precision generate --shape saw --freq 440 --rate 48000 --seconds 1 > output.raw

# Verify file size
# Header (64) + 48000 samples * 4 bytes/sample = 64 + 192000 = 192064
EXPECTED=192064
ACTUAL=$(stat -c%s output.raw)

if [ "$ACTUAL" -eq "$EXPECTED" ]; then
    echo "Verification Passed: File size is $ACTUAL bytes."
    rm output.raw
    exit 0
else
    echo "Verification Failed: Expected $EXPECTED bytes, got $ACTUAL bytes."
    exit 1
fi