#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: compare_flash_image.sh --stflash <path> [--serial <serial>] --addr <addr> --image <path> --out <path> [--under-reset]" >&2
}

under_reset=0
serial=""
stflash_args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --label)
      label="$2"
      shift 2
      ;;
    --stflash)
      stflash="$2"
      shift 2
      ;;
    --serial)
      serial="$2"
      shift 2
      ;;
    --addr)
      addr="$2"
      shift 2
      ;;
    --image)
      image="$2"
      shift 2
      ;;
    --out)
      out="$2"
      shift 2
      ;;
    --under-reset)
      under_reset=1
      shift
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

: "${stflash:?missing --stflash}"
: "${addr:?missing --addr}"
: "${image:?missing --image}"
: "${out:?missing --out}"
label="${label:-flash compare}"
if [[ -n "$serial" ]]; then
  stflash_args+=(--serial "$serial")
fi

size="$(wc -c < "$image")"
test "$size" -gt 0
rm -f "$out"
if [[ "$under_reset" -eq 1 ]]; then
  "$stflash" "${stflash_args[@]}" --connect-under-reset --freq=200K read "$out" "$addr" "$size"
else
  "$stflash" "${stflash_args[@]}" read "$out" "$addr" "$size"
fi
test -s "$out"
cmp -s "$image" "$out" || {
  echo "$label FAIL: device != $image" >&2
  exit 1
}
