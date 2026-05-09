#!/usr/bin/env bash
# Precision-DPW Tier-2 Kani verification runner (optional exploratory proofs)
# Runs Tier-2 runnable harnesses; does not run Tier-3 proof inventory.
# Run explicitly: scripts/verify_kani_tier2.sh
# Or via the main runner: RUN_TIER2=1 scripts/verify_kani.sh

set -euo pipefail
cd "$(dirname "$0")"
RUN_TIER2=1 bash ./verify_kani.sh
