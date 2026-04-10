#!/usr/bin/env bash
# Precision-DPW Protocol Level -1 verification runner (Kani)
# Normative: any mismatch or failed proof is non-conformant.

set -euo pipefail

REQUIRED_RUSTC_VERSION="1.91.1"
SUCCESS_TOKEN="VERIFICATION:- SUCCESSFUL"
DRY_RUN="${DRY_RUN:-0}"
ALLOW_DRY_RUN_PASS="${ALLOW_DRY_RUN_PASS:-0}"
RUN_HEAVY="${RUN_HEAVY:-0}"
DEFAULT_MAX_JOBS="${DEFAULT_MAX_JOBS:-4}"
SLOW_SECS="${SLOW_SECS:-360}"
KANI_LOG_DIR="kani_logs"
KANI_LOCK_DIR="$KANI_LOG_DIR/.verify_kani.lockdir"
# SSOT manifest: tier|crate|harness|target
HARNESS_MANIFEST='tier1|dpw4|proof_compute_x2_safe|lib
tier1|dpw4|proof_saturate_safe|lib
tier1|dpw4|proof_phase_u32_no_overflow|lib
tier1|dpw4|proof_phase_u32_fixed_to_u32_conversion|lib
tier1|dpw4|proof_sine_scale_no_overflow|lib
tier1|dpw4|proof_sine_to_i32_in_range|lib
tier1|dpw4|proof_sine_egress_bounded|lib
tier1|dpw4|proof_triangle_delta_clamp_identity_when_in_range|lib
tier1|dpw4|proof_triangle_delta_clamp_saturates_when_out_of_range|lib
tier1|dpw4|proof_triangle_z_update_is_saturating|lib
tier1|dpw4|proof_i256_sub_matches_spec|lib
tier1|dpw4|proof_i256_sar_in_range_matches_spec|lib
tier1|dpw4|proof_i256_sar_out_of_range_matches_spec|lib
tier1|dpw4|proof_i256_clamp_matches_spec|lib
tier1|dpw4|proof_spec_clamp_in_range_contract|lib
tier1|dpw4|proof_spec_clamp_out_of_range_contract|lib
tier1|dpw4|proof_spec_sar_sanity|lib
tier1|dpw4|proof_triangle_freeze_invariant|lib
tier1|dpw4|proof_triangle_freeze_egress_invariant|lib
tier1|geom-signal|proof_sqrt_no_panic|lib
tier1|geom-signal|proof_sin_cos_no_panic|lib
tier1|replay-core|proof_v0_wire_size_constants|lib
tier1|replay-core|proof_encode_header0_wire_layout_and_le|lib
tier1|replay-core|proof_encode_event_frame0_wire_layout_and_le|lib
tier2|dpw4|proof_i256_mul_u32_matches_spec|lib
tier2|geom-signal|proof_atan2_q1|lib
tier2|geom-signal|proof_atan2_q2|lib
tier2|geom-signal|proof_atan2_q3|lib
tier2|geom-signal|proof_atan2_q4|lib'

if [ -z "${KEEP_LOGS+x}" ]; then
    if [ "$RUN_HEAVY" = "1" ]; then
        KEEP_LOGS=1
    else
        KEEP_LOGS=0
    fi
else
    KEEP_LOGS="$KEEP_LOGS"
fi

if [ -n "${NO_COLOR:-}" ]; then
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    RESET=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[1;34m'
    RESET='\033[0m'
fi

print_header() {
    printf "\n${BLUE}=== %s ===${RESET}\n" "$1"
}

fail_msg() {
    printf "${RED}[FAILED]${RESET} %s\n" "$1" >&2
}

pass_msg() {
    printf "${GREEN}[PASSED]${RESET} %s\n" "$1"
}

warn_msg() {
    printf "${YELLOW}[WARN]${RESET} %s\n" "$1"
}

skip_msg() {
    printf "${YELLOW}[SKIPPED]${RESET} %s\n" "$1"
}

run_cmd() {
    if [ "$DRY_RUN" = "1" ]; then
        printf "DRY_RUN=1 -> "
        printf "%q " "$@"
        printf "\n"
        return 0
    fi

    "$@"
}

now_epoch() {
    date +%s
}

cpu_count() {
    if command -v nproc >/dev/null 2>&1; then
        nproc
        return
    fi

    if command -v getconf >/dev/null 2>&1; then
        getconf _NPROCESSORS_ONLN
        return
    fi

    if command -v sysctl >/dev/null 2>&1; then
        sysctl -n hw.ncpu 2>/dev/null || true
        return
    fi

    printf "1"
}

min_int() {
    local a="$1"
    local b="$2"
    if [ "$a" -le "$b" ]; then
        printf "%s" "$a"
    else
        printf "%s" "$b"
    fi
}

resolve_jobs() {
    if [ -n "${KANI_JOBS:-}" ]; then
        if ! [ "$KANI_JOBS" -ge 1 ] 2>/dev/null; then
            fail_msg "Invalid KANI_JOBS='$KANI_JOBS' (must be >= 1)"
            exit 1
        fi
        printf "%s" "$KANI_JOBS"
        return
    fi

    local cpus
    cpus=$(cpu_count)
    if ! [ "$cpus" -ge 1 ] 2>/dev/null; then
        cpus=1
    fi

    if ! [ "$DEFAULT_MAX_JOBS" -ge 1 ] 2>/dev/null; then
        fail_msg "Invalid DEFAULT_MAX_JOBS='$DEFAULT_MAX_JOBS' (must be >= 1)"
        exit 1
    fi

    min_int "$cpus" "$DEFAULT_MAX_JOBS"
}

check_cmd() {
    local name="$1"
    if ! command -v "$name" >/dev/null 2>&1; then
        fail_msg "Missing required command: $name"
        exit 1
    fi
}

check_package_exists() {
    local package="$1"
    local metadata

    if [ "$DRY_RUN" = "1" ]; then
        return 0
    fi

    metadata=$(cargo metadata --no-deps --format-version 1)
    if ! printf "%s" "$metadata" | grep -Eq "\"name\"[[:space:]]*:[[:space:]]*\"$package\""; then
        fail_msg "Workspace package not found: $package"
        exit 1
    fi
}

validate_success_token() {
    local log="$1"

    if ! grep -Fq "$SUCCESS_TOKEN" "$log"; then
        fail_msg "Missing success token in log: $log"
        printf "Expected token: %s\n" "$SUCCESS_TOKEN" >&2
        return 1
    fi

    if grep -Eq "^[[:space:]]*\\*\\*[[:space:]]*[1-9][0-9]*[[:space:]]+of[[:space:]]+[0-9]+[[:space:]]+failed" "$log"; then
        fail_msg "Verification failure summary detected in log: $log"
        return 1
    fi

    return 0
}

run_harness() {
    local label="$1"
    local package="$2"
    local harness="$3"
    local target_kind="$4"
    shift 4

    print_header "$label"

    local log="$KANI_LOG_DIR/${package}__${harness}.log"
    local cmd=(cargo kani -p "$package" --harness "$harness" --output-format terse)
    case "$target_kind" in
        lib)
            cmd+=(--lib)
            ;;
    esac
    if [ "$#" -gt 0 ]; then
        cmd+=("$@")
    fi

    if [ "$DRY_RUN" = "1" ]; then
        printf "Log file: %s\n" "$log"
        run_cmd "${cmd[@]}"
        skip_msg "$harness (dry run, non-normative)"
        return 0
    fi

    local start
    local end
    local elapsed
    start=$(now_epoch)

    "${cmd[@]}" 2>&1 | tee "$log"
    local cargo_exit="${PIPESTATUS[0]}"

    if [ "$cargo_exit" = "0" ] && validate_success_token "$log"; then
            end=$(now_epoch)
            elapsed=$((end - start))
            pass_msg "$harness (${elapsed}s)"
            if [ "$elapsed" -gt "$SLOW_SECS" ]; then
                warn_msg "$harness exceeded SLOW_SECS=${SLOW_SECS}s (${elapsed}s)"
            fi
            if [ "$KEEP_LOGS" != "1" ]; then
                rm -f "$log"
            else
                printf "Log kept: %s\n" "$log"
            fi
            return 0
    fi

    end=$(now_epoch)
    elapsed=$((end - start))
    fail_msg "$harness (${elapsed}s)"
    printf "Failure log: %s\n" "$log" >&2
    if [ -f "$log" ]; then
        printf "%s\n" "--- log tail ---" >&2
        tail -n 20 "$log" >&2 || true
        printf "%s\n" "--- end log tail ---" >&2
    fi
    exit 1
}

print_header "Pre-flight: Environment"
check_cmd cargo
check_cmd rustc

RUSTC_VERSION_RAW=$(rustc --version)
RUSTC_VERSION=$(printf "%s" "$RUSTC_VERSION_RAW" | awk '{print $2}')
if [ "$RUSTC_VERSION" != "$REQUIRED_RUSTC_VERSION" ]; then
    fail_msg "Toolchain mismatch. Required rustc $REQUIRED_RUSTC_VERSION, found: $RUSTC_VERSION_RAW"
    exit 1
fi
pass_msg "Toolchain locked to rustc $REQUIRED_RUSTC_VERSION"

if [ "$DRY_RUN" = "1" ]; then
    printf "DRY_RUN=1 -> cargo kani --version\n"
    skip_msg "cargo kani plugin check (dry run)"
else
    if ! cargo kani --version >/dev/null 2>&1; then
        fail_msg "cargo kani is not available or not functioning"
        exit 1
    fi
    pass_msg "cargo kani plugin available"
fi

check_package_exists "dpw4"
check_package_exists "geom-signal"
check_package_exists "replay-core"
pass_msg "Required workspace packages present"

JOBS=$(resolve_jobs)
mkdir -p "$KANI_LOG_DIR"
if ! mkdir "$KANI_LOCK_DIR" 2>/dev/null; then
    fail_msg "Another verify_kani.sh run appears active (lock: $KANI_LOCK_DIR)"
    fail_msg "Wait for it to finish or remove stale lock dir if no process is running."
    exit 1
fi
trap 'rmdir "$KANI_LOCK_DIR" >/dev/null 2>&1 || true' EXIT

print_header "Kani Formal Verification (Protocol Level -1)"
printf "Tier mode: %s\n" "$( [ "$RUN_HEAVY" = "1" ] && printf "Tier-2 (heavy enabled)" || printf "Tier-1 (fast default)" )"
printf "Shard jobs (atan2): %s\n" "$JOBS"
printf "KEEP_LOGS=%s\n" "$KEEP_LOGS"
printf "SLOW_SECS=%s\n" "$SLOW_SECS"
if [ "$DRY_RUN" = "1" ]; then
    warn_msg "DRY_RUN active: no harnesses executed; this output is NON-NORMATIVE."
fi

TOTAL_START=$(now_epoch)

while IFS='|' read -r tier package harness target_kind; do
    [ -z "${tier:-}" ] && continue
    if [ "$tier" = "tier2" ] && [ "$RUN_HEAVY" != "1" ]; then
        warn_msg "Skipping heavy harness $harness (set RUN_HEAVY=1 to enable)"
        continue
    fi

    run_harness "Harness: $harness" "$package" "$harness" "$target_kind"
done <<EOF
$HARNESS_MANIFEST
EOF

TOTAL_END=$(now_epoch)
TOTAL_ELAPSED=$((TOTAL_END - TOTAL_START))


print_header "Summary"
if [ "$DRY_RUN" != "1" ]; then
    pass_msg "Kani verification complete (${TOTAL_ELAPSED}s total)"
else
    skip_msg "Kani verification summary (dry run)"
fi



if [ "$DRY_RUN" = "1" ]; then
    warn_msg "NON-NORMATIVE summary: no proofs were executed."
    if [ "$ALLOW_DRY_RUN_PASS" = "1" ]; then
        warn_msg "ALLOW_DRY_RUN_PASS=1 set; returning exit code 0 for tooling compatibility."
        exit 0
    fi
    fail_msg "DRY_RUN default policy: returning exit code 2 to prevent false green evidence."
    exit 2
fi
