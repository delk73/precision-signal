# NON-NORMATIVE / EXPERIMENTAL

Status: HOSTILE-BOARD ASSUMPTIONS APPLY

This procedure is exploratory only. It does not establish trusted hardware,
release surface, verification authority, or production readiness.

## Purpose

Define a minimal, auditable bring-up procedure for an unknown-source
BeagleBone Black (BBB) using an offline-first posture, explicit trust-boundary
reduction, and retained evidence. The output of this procedure is a constrained
experimental surface only.

## Threat / Trust Assumptions

- The BBB is unknown-provenance hardware and may be modified, damaged, or
  hostile.
- Existing onboard eMMC state is untrusted and must not be treated as a known
  baseline.
- Initial bring-up must not depend on ambient network connectivity.
- Every action must either reduce uncertainty or reduce exposed surface.
- Unknowns remain unknown until evidence is retained.
- PASS at T5 permits limited experimental use only. It does not imply the board
  is trusted.

## Phase Plan T0-T5

### T0 - Intake / Physical Provenance

Goal: document what arrived before power-on.

Tasks
- Photograph board front, back, packaging, and included accessories before any
  cleaning or modification.
- Record seller/source, listing text if available, shipment date, packaging
  condition, and any chain-of-custody notes known to the operator.
- Inspect for visible damage, rework, missing parts, corrosion, flux residue,
  altered jumpers, unexpected daughterboards, inserted storage media, or
  hand-soldered modifications.
- Record all visible markings, serial labels, PCB revision text, and major IC
  date codes if readable.
- Record available power input options actually present on the board and cable
  set.
- Assign a local evidence ID, for example `BBB-001`, and use it on all notes
  and photo filenames.
- All retained artifacts must include the evidence ID in filenames.

Retained Evidence
- Photo set named with local evidence ID
- Intake notes
- Board revision and markings log

Acceptance Criteria
- Intake record completed before first power-on
- Board uniquely identified in notes
- No immediate red flags that require quarantine before power application

Decision
- PASS: continue to T1
- HOLD: incomplete provenance record or unclear markings
- FAIL: quarantine for severe physical anomaly, tamper evidence, or unsafe power condition

### T1 - Imaging Strategy

Goal: choose a known-good boot medium and avoid trusting existing onboard state.

Tasks
- Use a freshly written microSD card as the preferred boot medium.
- Do not rely on existing eMMC contents for first boot.
- Obtain an official minimal image on the Omarchy laptop and record the exact
  source URL, filename, release identifier, and published checksum.
- Verify the downloaded image checksum before writing media.
- Identify the target microSD device path explicitly and confirm it is not a
  host system disk.
- Write the image with a single auditable command and record both the command
  and resulting device mapping.
- Eject and reinsert the card if needed to confirm the expected partition table
  appears on the written medium.

Concrete Host Commands
```bash
# Record source and checksum values exactly as obtained from the publisher.
export BBB_IMAGE_URL='<official-image-url>'
export BBB_IMAGE_FILE='<image-file.xz>'
export BBB_IMAGE_SHA256='<published-sha256>'

sha256sum "$BBB_IMAGE_FILE"
lsblk -o NAME,SIZE,MODEL,SERIAL,TRAN,RM,MOUNTPOINT
xzcat "$BBB_IMAGE_FILE" | sudo dd of=/dev/sdX bs=4M conv=fsync status=progress
# If image is not compressed, record that explicitly and use dd directly.
sync
lsblk -o NAME,SIZE,FSTYPE,LABEL,MOUNTPOINT /dev/sdX
```

Notes
- Replace `/dev/sdX` with the confirmed removable device path.
- If the image is compressed, record the exact decompression or imaging tool
  used; do not omit that step from evidence.
- Do not write to eMMC in this phase.

Retained Evidence
- Image source record
- Checksum verification result
- Imaging transcript or command block
- Device mapping snapshot from the Omarchy laptop

Acceptance Criteria
- Image provenance recorded
- Checksum verified against publisher value
- Fresh boot media prepared
- No ambiguity remains about target device or written image

Decision
- PASS: continue to T2
- HOLD: missing checksum, uncertain device mapping, or incomplete transcript
- FAIL: checksum mismatch or write target uncertainty that risks host data loss

### T2 - First Boot in Isolation

Goal: achieve shell access without granting ambient network trust.

Tasks
- Keep Ethernet disconnected.
- Prefer UART serial console as the operator path.
- If UART is not available, use the narrowest local-only path and record why
  UART was unavailable.
- Do not rely on USB gadget networking during this phase. If USB gadget
  networking enumerates automatically, document it and avoid using it as the
  operator path.
- Boot from the prepared microSD card and record whether boot selection appears
  to prefer microSD over eMMC.
- Capture boot messages, login prompt appearance, kernel version, and root
  filesystem or boot-source observations.
- Record all observed interfaces immediately after login.
- Detect presence of USB gadget interfaces (e.g., `usb0`) and record their
  state even if unused.

Concrete Target Commands
```bash
uname -a
cat /etc/os-release
findmnt -no SOURCE /
findmnt -no SOURCE /boot || true
ip -brief link
ip -brief addr
lsblk -o NAME,MOUNTPOINT,SOURCE
```

Retained Evidence
- Boot log
- Console transcript
- Observed interface list
- Note identifying the actual operator access path
- Block device mapping confirming rootfs source

Acceptance Criteria
- Stable shell achieved
- No intentional network connectivity used
- Operator access path documented
- Boot source observations recorded
- Boot confirmed from microSD (rootfs device not eMMC)

Decision
- PASS: continue to T3
- HOLD: shell unstable, uncertain boot source, or operator path not fully documented
- FAIL: no stable local shell or unexplained boot anomaly that blocks isolated operation

### T3 - Surface Reduction

Goal: reduce exposed services and establish a minimal experimental boundary.

Tasks
- Inventory enabled services before making changes.
- Inventory network interfaces and assigned addresses.
- Inventory listening sockets before and after changes.
- Disable or stop network-facing services not required for isolated operation.
- Bring down network interfaces not required for the chosen local operator
  boundary.
- Prefer `UART-only` as the initial operator boundary.
- If `UART-only` is not achieved, state the fallback local-only boundary
  explicitly and justify it.
- Declare operator boundary in a single explicit line:
  `Operator Boundary: UART-only` or `Operator Boundary: local-only (justified)`
- Run credential change (`passwd`) and record outcome (changed or not
  applicable).
- Record all persistent configuration changes made on the target.

Concrete Target Commands
```bash
systemctl list-unit-files --state=enabled
ip link
ip addr
ss -tulpen

# Examples; adjust only after observing actual units and interfaces.
sudo systemctl disable --now connman.service || true
sudo systemctl disable --now avahi-daemon.service || true
sudo ip link set eth0 down || true
sudo ip link set usb0 down || true
passwd

systemctl list-unit-files --state=enabled
ip link
ip addr
ss -tulpen
```

Retained Evidence
- `systemctl` enabled-unit snapshot
- `ip link` and `ip addr` snapshots
- `ss -tulpen` snapshots before and after reduction
- Credential-change result (changed / not applicable)
- Persistent configuration change log
- Explicit operator boundary declaration line

Acceptance Criteria
- No active network interfaces hold addresses for the accepted operator boundary
- No unnecessary listeners remain
- Default credentials removed if present
- Operator boundary explicitly declared in required format

Decision
- PASS: continue to T4
- HOLD: uncertainty remains about services, interfaces, or credential state
- FAIL: network surface cannot be reduced to the documented boundary

### T4 - Baseline Characterization

Goal: establish a reproducible pre-experiment baseline.

Tasks
- Record OS release, kernel version, boot source, mounted filesystems, and
  `dmesg` warnings of interest.
- Record board identity from available system interfaces.
- Snapshot boot configuration files relevant to the selected boot path.
- Record a package manifest only if the image already provides one and it is
  useful for later comparison.
- Hash retained configuration artifacts where practical.
- Hash at least one boot configuration artifact or explicitly record that none
  are present.
- Call out any major unknowns explicitly instead of normalizing them away.

Concrete Target Commands
```bash
uname -a
cat /etc/os-release
findmnt
dmesg -T | tail -n 200
cat /proc/device-tree/model 2>/dev/null || true
cat /proc/device-tree/serial-number 2>/dev/null || true
ls /boot /boot/firmware 2>/dev/null || true
sha256sum /boot/uEnv.txt 2>/dev/null || true
sha256sum /boot/extlinux/extlinux.conf 2>/dev/null || true
dpkg-query -W 2>/dev/null || true
opkg list-installed 2>/dev/null || true
rpm -qa 2>/dev/null || true
```

Retained Evidence
- Baseline summary
- Config file copies or hashes
- Hardware identity notes
- Explicit unknowns list

Acceptance Criteria
- Baseline is sufficient to compare later changes
- Major unknowns are called out explicitly
- Retained artifacts are named and attributable to this board instance
- At least one config artifact hashed or explicitly marked absent

Decision
- PASS: continue to T5
- HOLD: baseline incomplete or identity data contradictory
- FAIL: unexplained anomalies make limited experimentation unsafe or non-repeatable

### T5 - Decision Gate

Goal: decide whether the board is acceptable for limited experimental use.

Decision Outcomes
- PASS: safe for constrained experimental role
- HOLD: more inspection needed
- FAIL: quarantine; do not integrate

PASS Criteria
- Known-good boot medium used
- Stable shell achieved
- Network surface minimized
- Operator boundary explicit
- No unexplained physical or boot anomalies
- Evidence retained for T0 through T4

Required Decision Record
- Board evidence ID
- Final decision: PASS, HOLD, or FAIL
- Decision date
- Named operator
- Short rationale tied to retained evidence
- Explicit statement that PASS does not imply trusted hardware

## Exact Evidence Checklist

Use [bringup_evidence_checklist.md](templates/bringup_evidence_checklist.md)
as the operator worksheet. Required retained items are:

- Intake: photos, source notes, markings, revision, visible anomaly notes
- Image provenance: source URL, version, checksum source, checksum result,
  write command, removable-device mapping
- First boot: boot log, console transcript, observed interfaces, boot-source note
- Services/interfaces: enabled-unit list, `ip` snapshots, `ss -tulpen` snapshots,
  credential-change note, persistent config change log
- Baseline capture: OS/kernel record, mount list, identity notes, boot-config
  hashes or copies, unknowns list
- Decision gate: PASS/HOLD/FAIL decision record with rationale

## Acceptance Criteria By Phase

| Phase | Minimum evidence to proceed | Proceed only if |
| --- | --- | --- |
| T0 | intake notes + photo set + board ID | no immediate quarantine condition |
| T1 | source record + checksum result + write transcript | boot media provenance is clear |
| T2 | boot log + console transcript + interface snapshot | stable isolated shell exists |
| T3 | service snapshot + interface snapshot + listener snapshot | operator boundary is explicit and reduced |
| T4 | baseline summary + config hashes/copies + identity notes | later comparisons are possible |
| T5 | decision record referencing T0-T4 evidence | limited experimental role is justified or rejected |

## Abort Conditions

- Electrical damage, overheating, smell, or other unsafe power behavior
- Evidence of tampering or rework that cannot be explained
- Inability to identify the board revision or boot source with reasonable confidence
- Checksum mismatch on the selected boot image
- Requirement for network connectivity before surface reduction is complete
- Inability to remove default credentials or reduce the operator boundary
- Contradictory observations that prevent an auditable baseline

## Promotion Criteria To `docs/hardware/`

Do not promote this procedure solely because a single board passed T5. Promotion
requires:

- repeated bring-up on the same documented BBB revision
- stable, repeatable evidence across multiple sessions or boards
- a clear hardware-routing need under `docs/hardware/`
- no change in authority claims for release, spec, or verification surfaces
- review that converts hostile-board assumptions into a narrower supported scope,
  if justified by evidence
