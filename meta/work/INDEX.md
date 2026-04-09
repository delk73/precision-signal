# Work Index

Current packet:
- 1.6.0 retained release-bundle preparation

Current branch:
- release/1.6.0-cli-comp

Last completed packet:
- 2026-04-09_1.6.0_release-readiness.md

Next packet after current:
- 1.6.0 formal release cut

Current blocker:
- docs/verification/releases/1.6.0 missing; make release-bundle-check VERSION=1.6.0 fails

Rule:
- planning files are packet-scoped, not perpetual
- only one active packet file exists at a time: meta/work/active/CURRENT.md
- when a packet changes or completes, archive it under meta/work/archive/YYYY-MM-DD_<packet-name>.md and start a new CURRENT.md
