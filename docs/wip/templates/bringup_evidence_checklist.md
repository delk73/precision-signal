# NON-NORMATIVE / EXPERIMENTAL

Use this checklist to retain evidence for hostile-board bring-up. Do not mark a
phase complete without corresponding artifacts or notes.

## Intake

- [ ] Local evidence ID assigned
- [ ] Front photo captured
- [ ] Back photo captured
- [ ] Packaging/accessories photo captured
- [ ] Seller/source and listing text recorded
- [ ] Packaging condition recorded
- [ ] Visible markings and board revision recorded
- [ ] Visible anomaly notes recorded
- [ ] Power input options recorded
- [ ] T0 decision recorded: PASS / HOLD / FAIL

## Image Provenance

- [ ] Official image source URL recorded
- [ ] Image filename/version recorded
- [ ] Published checksum recorded
- [ ] Checksum verification result retained
- [ ] Target removable device mapping retained
- [ ] Exact imaging command retained
- [ ] Post-write partition or filesystem observation retained
- [ ] T1 decision recorded: PASS / HOLD / FAIL

## First Boot

- [ ] Ethernet remained disconnected
- [ ] Operator access path recorded
- [ ] Boot log retained
- [ ] Console transcript retained
- [ ] Kernel version recorded
- [ ] Root filesystem or boot source recorded
- [ ] Boot confirmed from microSD (not eMMC)
- [ ] Block device mapping retained (rootfs source)
- [ ] Observed interfaces retained
- [ ] T2 decision recorded: PASS / HOLD / FAIL

## Services / Interfaces

- [ ] Enabled-unit snapshot retained
- [ ] `ip link` snapshot retained
- [ ] `ip addr` snapshot retained
- [ ] `ss -tulpen` snapshot retained before reduction
- [ ] Listener snapshot retained after reduction
- [ ] Credential-change result recorded (changed / not applicable)
- [ ] Persistent config changes recorded
- [ ] Operator boundary declared in required format
- [ ] T3 decision recorded: PASS / HOLD / FAIL

## Baseline Capture

- [ ] OS release recorded
- [ ] Kernel version recorded
- [ ] Mount list retained
- [ ] `dmesg` warnings of interest recorded
- [ ] Board identity notes retained
- [ ] Boot config file copies or hashes retained
- [ ] At least one config artifact hashed or explicitly marked absent
- [ ] Package manifest retained or marked not useful
- [ ] Explicit unknowns listed
- [ ] T4 decision recorded: PASS / HOLD / FAIL

## Decision Gate

- [ ] Known-good boot medium confirmed
- [ ] Stable shell confirmed
- [ ] Network surface minimized
- [ ] Operator boundary confirmed
- [ ] No unexplained physical or boot anomaly remains
- [ ] Evidence set for T0-T4 retained
- [ ] Final decision recorded: PASS / HOLD / FAIL
- [ ] Decision rationale recorded
- [ ] PASS explicitly limited to constrained experimental use only
