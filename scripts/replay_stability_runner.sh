#!/usr/bin/env bash
set -euo pipefail

cycles=5
serial="/dev/ttyACM0"
output_root="artifacts/stability_envelope"
timeout="10"
reset_mode="stlink"
stflash="st-flash"

usage() {
  cat <<'EOF'
Usage: scripts/replay_stability_runner.sh [options]

Options:
  --cycles <count>         Number of capture->record->replay cycles (default: 5)
  --serial <device>        Serial device for STM32 UART capture (default: /dev/ttyACM0)
  --output-root <dir>      Parent directory for session evidence roots (default: artifacts/stability_envelope)
  --timeout <seconds>      Capture timeout passed to repeat_capture.py (default: 10)
  --reset-mode <mode>      Reset mode for repeat_capture.py (default: stlink)
  --stflash <path>         st-flash executable path/name (default: st-flash)
  --help                   Show this help text
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cycles)
      cycles="${2:?missing value for --cycles}"
      shift 2
      ;;
    --serial)
      serial="${2:?missing value for --serial}"
      shift 2
      ;;
    --output-root)
      output_root="${2:?missing value for --output-root}"
      shift 2
      ;;
    --timeout)
      timeout="${2:?missing value for --timeout}"
      shift 2
      ;;
    --reset-mode)
      reset_mode="${2:?missing value for --reset-mode}"
      shift 2
      ;;
    --stflash)
      stflash="${2:?missing value for --stflash}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'error: unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$cycles" in
  ''|*[!0-9]*)
    printf 'error: --cycles must be a positive integer\n' >&2
    exit 2
    ;;
esac
if [[ "$cycles" -le 0 ]]; then
  printf 'error: --cycles must be > 0\n' >&2
  exit 2
fi

case "$reset_mode" in
  manual|stlink) ;;
  *)
    printf 'error: --reset-mode must be manual or stlink\n' >&2
    exit 2
    ;;
esac

session_id_base="$(date -u +%Y%m%dT%H%M%SZ)"
session_id="$session_id_base"
session_root="${output_root%/}/${session_id}"
session_suffix=0
while [[ -e "$session_root" ]]; do
  session_suffix=$((session_suffix + 1))
  session_id="${session_id_base}_$(printf '%02d' "$session_suffix")"
  session_root="${output_root%/}/${session_id}"
done
mkdir -p "$session_root"

manifest_path="$session_root/manifest.tsv"
status_path="$session_root/session_status.txt"

printf 'cycle\tcapture_csv\tcapture_status\tcapture_sha256\trecord_artifact\treplay_artifact\tstatus\tfailure_class\n' > "$manifest_path"

write_session_status() {
  local status="$1"
  local completed="$2"
  local failure_cycle="$3"
  local failure_class="$4"
  cat > "$status_path" <<EOF
session_id=$session_id
session_root=$session_root
status=$status
cycles_requested=$cycles
cycles_completed=$completed
failure_cycle=$failure_cycle
failure_class=$failure_class
EOF
}

append_manifest_row() {
  local cycle_id="$1"
  local capture_csv="$2"
  local capture_status="$3"
  local capture_sha="$4"
  local record_artifact="$5"
  local replay_artifact="$6"
  local status="$7"
  local failure_class="$8"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$cycle_id" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "$status" "$failure_class" \
    >> "$manifest_path"
}

finalize_stderr_file() {
  local path="$1"
  if [[ -f "$path" && ! -s "$path" ]]; then
    rm -f "$path"
  fi
}

run_to_files() {
  local stdout_path="$1"
  local stderr_path="$2"
  shift 2
  set +e
  "$@" >"$stdout_path" 2>"$stderr_path"
  local rc=$?
  set -e
  finalize_stderr_file "$stderr_path"
  return "$rc"
}

fail_cycle() {
  local cycle_id="$1"
  local cycle_dir="$2"
  local capture_csv="${3:-}"
  local capture_status="${4:-}"
  local capture_sha="${5:-}"
  local record_artifact="${6:-}"
  local replay_artifact="${7:-}"
  local failure_class="$8"
  local completed="$9"
  cat > "$cycle_dir/failure.txt" <<EOF
cycle=$cycle_id
failure_class=$failure_class
EOF
  append_manifest_row "$cycle_id" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "FAIL" "$failure_class"
  write_session_status "FAIL" "$completed" "$cycle_id" "$failure_class"
  printf 'FAIL cycle=%s class=%s\n' "$cycle_id" "$failure_class" >&2
  exit 1
}

require_single_exact_line() {
  local file_path="$1"
  local expected_line="$2"
  local failure_class="$3"
  local count
  count="$(grep -Fxc "$expected_line" "$file_path" || true)"
  if [[ "$count" != "1" ]]; then
    return 1
  fi
  return 0
}

extract_artifact_path() {
  local file_path="$1"
  mapfile -t artifact_lines < <(grep '^ARTIFACT: ' "$file_path" || true)
  if [[ "${#artifact_lines[@]}" -ne 1 ]]; then
    return 1
  fi
  local artifact_path="${artifact_lines[0]#ARTIFACT: }"
  if [[ -z "$artifact_path" ]]; then
    return 1
  fi
  printf '%s\n' "$artifact_path"
}

verify_result_block() {
  local file_path="$1"
  local command_name="$2"
  local target_path="$3"
  local cycle_id="$4"
  local cycle_dir="$5"
  local capture_csv="$6"
  local capture_status="$7"
  local capture_sha="$8"
  local record_artifact="$9"
  local replay_artifact="${10}"
  local completed_before_failure="${11}"
  local line_count

  line_count="$(awk 'END { print NR }' "$file_path")"
  if [[ "$line_count" != "7" ]]; then
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_result_block_line_count_invalid" "$completed_before_failure"
  fi
  require_single_exact_line "$file_path" "RESULT: PASS" "${command_name}_result_not_pass" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_result_not_pass" "$completed_before_failure"
  require_single_exact_line "$file_path" "COMMAND: $command_name" "${command_name}_command_field_invalid" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_command_field_invalid" "$completed_before_failure"
  require_single_exact_line "$file_path" "TARGET: $target_path" "${command_name}_target_field_invalid" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_target_field_invalid" "$completed_before_failure"
  require_single_exact_line "$file_path" "MODE: runtime_mode" "${command_name}_mode_field_invalid" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_mode_field_invalid" "$completed_before_failure"
  require_single_exact_line "$file_path" "EQUIVALENCE: exact" "${command_name}_equivalence_not_exact" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_equivalence_not_exact" "$completed_before_failure"
  require_single_exact_line "$file_path" "FIRST_DIVERGENCE: none" "${command_name}_first_divergence_not_none" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_first_divergence_not_none" "$completed_before_failure"
  if ! extract_artifact_path "$file_path" >/dev/null; then
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_artifact_field_invalid" "$completed_before_failure"
  fi
}

verify_artifact_dir() {
  local artifact_dir="$1"
  local stdout_file="$2"
  local command_name="$3"
  local cycle_id="$4"
  local cycle_dir="$5"
  local capture_csv="$6"
  local capture_status="$7"
  local capture_sha="$8"
  local record_artifact="$9"
  local replay_artifact="${10}"
  local completed_before_failure="${11}"

  [[ -d "$artifact_dir" ]] || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_artifact_dir_missing" "$completed_before_failure"
  [[ -f "$artifact_dir/result.txt" ]] || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_artifact_result_missing" "$completed_before_failure"
  [[ -f "$artifact_dir/trace.json" ]] || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_artifact_trace_missing" "$completed_before_failure"
  [[ -f "$artifact_dir/meta.json" ]] || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_artifact_meta_missing" "$completed_before_failure"
  cmp -s "$stdout_file" "$artifact_dir/result.txt" || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "${command_name}_result_stdout_mismatch" "$completed_before_failure"
}

write_session_status "IN_PROGRESS" "0" "-" "-"

for cycle_num in $(seq 1 "$cycles"); do
  cycle_id="$(printf 'cycle_%02d' "$cycle_num")"
  cycle_dir="$session_root/$cycle_id"
  mkdir -p "$cycle_dir"

  capture_stdout="$cycle_dir/capture.stdout.txt"
  capture_stderr="$cycle_dir/capture.stderr.txt"
  capture_status_file="$cycle_dir/capture.status.txt"
  capture_sha_file="$cycle_dir/capture.sha256.txt"
  capture_csv_file="$cycle_dir/run_01.csv"
  capture_csv_path_file="$cycle_dir/capture_csv_path.txt"
  record_stdout="$cycle_dir/record.stdout.txt"
  record_stderr="$cycle_dir/record.stderr.txt"
  record_artifact_path_file="$cycle_dir/record_artifact_path.txt"
  replay_stdout="$cycle_dir/replay.stdout.txt"
  replay_stderr="$cycle_dir/replay.stderr.txt"
  replay_artifact_path_file="$cycle_dir/replay_artifact_path.txt"

  printf '%s\n' "$capture_csv_file" > "$capture_csv_path_file"

  if ! env SERIAL="$serial" python3 scripts/repeat_capture.py \
      --contract csv \
      --runs 1 \
      --reset-mode "$reset_mode" \
      --stflash "$stflash" \
      --artifacts-dir "$cycle_dir" \
      --timeout "$timeout" \
      >"$capture_stdout" 2>"$capture_stderr"; then
    finalize_stderr_file "$capture_stderr"
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv_file" "" "" "" "" "capture_command_failed" "$((cycle_num - 1))"
  fi
  finalize_stderr_file "$capture_stderr"

  capture_status="$(grep -Fx 'STATE,CAPTURE_DONE,138' "$capture_stdout" || true)"
  if [[ -z "$capture_status" ]]; then
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv_file" "" "" "" "" "capture_status_missing_or_invalid" "$((cycle_num - 1))"
  fi
  status_count="$(grep -Fxc 'STATE,CAPTURE_DONE,138' "$capture_stdout" || true)"
  if [[ "$status_count" != "1" ]]; then
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv_file" "" "" "" "" "capture_status_count_invalid" "$((cycle_num - 1))"
  fi
  printf '%s\n' "$capture_status" > "$capture_status_file"

  [[ -f "$capture_csv_file" ]] || \
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "" "" "" "capture_csv_missing" "$((cycle_num - 1))"
  capture_sha="$(sha256sum "$capture_csv_file" | awk '{print $1}')"
  printf '%s\n' "$capture_sha" > "$capture_sha_file"

  if ! run_to_files "$record_stdout" "$record_stderr" cargo run -q -p dpw4 --features cli --bin precision -- \
      record "$capture_csv_file" --mode runtime_mode; then
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "$capture_sha" "" "" "record_command_failed" "$((cycle_num - 1))"
  fi

  verify_result_block "$record_stdout" "record" "$capture_csv_file" "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "$capture_sha" "" "" "$((cycle_num - 1))"
  record_artifact="$(extract_artifact_path "$record_stdout")"
  printf '%s\n' "$record_artifact" > "$record_artifact_path_file"
  verify_artifact_dir "$record_artifact" "$record_stdout" "record" "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "$capture_sha" "$record_artifact" "" "$((cycle_num - 1))"

  if ! run_to_files "$replay_stdout" "$replay_stderr" cargo run -q -p dpw4 --features cli --bin precision -- \
      replay "$record_artifact" --mode runtime_mode; then
    fail_cycle "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "$capture_sha" "$record_artifact" "" "replay_command_failed" "$((cycle_num - 1))"
  fi

  verify_result_block "$replay_stdout" "replay" "$record_artifact" "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "$capture_sha" "$record_artifact" "" "$((cycle_num - 1))"
  replay_artifact="$(extract_artifact_path "$replay_stdout")"
  printf '%s\n' "$replay_artifact" > "$replay_artifact_path_file"
  verify_artifact_dir "$replay_artifact" "$replay_stdout" "replay" "$cycle_id" "$cycle_dir" "$capture_csv_file" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "$((cycle_num - 1))"

  append_manifest_row "$cycle_id" "$capture_csv_file" "$capture_status" "$capture_sha" "$record_artifact" "$replay_artifact" "PASS" "-"
  write_session_status "IN_PROGRESS" "$cycle_num" "-" "-"
done

write_session_status "PASS" "$cycles" "-" "-"
printf 'PASS session=%s cycles=%s\n' "$session_root" "$cycles"
