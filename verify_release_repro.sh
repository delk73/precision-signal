#!/usr/bin/env bash
# verify_release_repro.sh — Δ-04 dual-build hash canary
# Usage: bash verify_release_repro.sh
# Exits 0 if both release builds produce identical binaries.
# Exits 1 if hashes differ (non-deterministic build detected).
# Optional:
#   RELEASE_EVIDENCE_DIR=docs/verification/releases/<version>/ bash verify_release_repro.sh
#   Writes a retained supporting evidence record into the canonical release
#   evidence directory.

set -euo pipefail

BINARY="${REPRO_BINARY:-release/sig-util}"
RELEASE_EVIDENCE_DIR="${RELEASE_EVIDENCE_DIR:-}"
SOURCE_DATE_EPOCH_VALUE="${SOURCE_DATE_EPOCH:-$(git show -s --format=%ct HEAD)}"

echo "=== Δ-04 Release Reproducibility Check ==="
echo "Toolchain: $(rustc -Vv | head -1)"
echo "Cargo:     $(cargo -V)"
echo "Binary:    ${BINARY}"
echo "SOURCE_DATE_EPOCH: ${SOURCE_DATE_EPOCH_VALUE}"
echo ""

# Clean slate — never reuse prior artifacts
rm -rf target_a target_b

echo "[1/2] First build → target_a/"
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH_VALUE}" \
CARGO_TARGET_DIR=target_a \
cargo build --release -p dpw4 --features cli --locked

echo ""
echo "[2/2] Second build → target_b/"
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH_VALUE}" \
CARGO_TARGET_DIR=target_b \
cargo build --release -p dpw4 --features cli --locked

echo ""
echo "=== Hashing artifacts ==="
hash_a="$(sha256sum "target_a/${BINARY}" | awk '{print $1}')"
hash_b="$(sha256sum "target_b/${BINARY}" | awk '{print $1}')"

echo "Build A: ${hash_a}  target_a/${BINARY}"
echo "Build B: ${hash_b}  target_b/${BINARY}"

if [[ -n "${RELEASE_EVIDENCE_DIR}" ]]; then
    mkdir -p "${RELEASE_EVIDENCE_DIR}"
fi

if cmp -s "target_a/${BINARY}" "target_b/${BINARY}"; then
    echo ""
    echo "✅ PASS — builds are bit-for-bit identical: ${hash_a}"
    if [[ -n "${RELEASE_EVIDENCE_DIR}" ]]; then
        cat > "${RELEASE_EVIDENCE_DIR}/release_reproducibility.txt" <<EOF
build_reproducibility_status=PASS
toolchain=$(rustc -Vv | head -1)
cargo=$(cargo -V)
binary=${BINARY#release/}
source_date_epoch=${SOURCE_DATE_EPOCH_VALUE}
build_a_sha256=${hash_a}
build_b_sha256=${hash_b}
comparison=bit-identical
script=verify_release_repro.sh
EOF
    fi
    exit 0
else
    echo ""
    echo "❌ FAIL — build outputs differ:"
    echo "A: ${hash_a}"
    echo "B: ${hash_b}"
    if [[ -n "${RELEASE_EVIDENCE_DIR}" ]]; then
        cat > "${RELEASE_EVIDENCE_DIR}/release_reproducibility.txt" <<EOF
build_reproducibility_status=FAIL
toolchain=$(rustc -Vv | head -1)
cargo=$(cargo -V)
binary=${BINARY#release/}
source_date_epoch=${SOURCE_DATE_EPOCH_VALUE}
build_a_sha256=${hash_a}
build_b_sha256=${hash_b}
comparison=diverged
script=verify_release_repro.sh
EOF
    fi
    exit 1
fi
