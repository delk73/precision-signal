#!/usr/bin/env bash
# Precision-DPW Tier-2 Kani verification runner (optional / nightly)
# These harnesses are correct but exceed practical CI time budgets.
# Run explicitly: ./verify_kani_tier2.sh
# Or via the main runner: RUN_HEAVY=1 ./verify_kani.sh

set -euo pipefail
cd "$(dirname "$0")"
RUN_HEAVY=1 bash ./verify_kani.sh
