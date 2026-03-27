# NON-NORMATIVE / EXPERIMENTAL

Board Evidence ID: BBB-001
Board: BeagleBone Black
Date Opened: 2026-03-26
Status: active

Use this worksheet to retain evidence for hostile-board bring-up. Do not mark a
phase complete without corresponding artifacts or notes.

## Intake

- [x] Local evidence ID assigned
- [x] Front photo captured
- [x] Back photo captured
- [x] Packaging/accessories photo captured
- [x] Seller/source and listing text recorded
- [x] Packaging condition recorded
- [x] Visible markings and board revision recorded
- [x] Visible anomaly notes recorded
- [x] Power input options recorded
- [ ] T0 decision recorded: PASS / HOLD / FAIL

Evidence Notes
- Evidence ID selected as `BBB-001`.
- Seller/source evidence from attached context:
  `DigiKey`, order `97939028`, invoice `122220810`.
- Product listing evidence from attached context:
  DigiKey part `2820-102110420-ND`, manufacturer `BeagleBoard`,
  manufacturer part `102110420`, description `BEAGLEBONE BLACK AM3358`,
  platform `BeagleBone Black`, utilized IC/part `AM3358BZCZ`.
- Shipment evidence from attached context shows shipment marked `Shipped`
  with FedEx Ground tracking `517546930907`.
- Operator reports physical board front/back photos and packaging/accessories
  photos were recorded.
- Image evidence is retained in the chat attachment context for this bring-up.
- Operator reports packaging condition was normal.
- Operator reports no anomaly in the package or board.
- Board front/back photos in chat context confirm standard BBB connector and
  layout population with no obvious missing ports or add-on boards.
- Readable silkscreen from the attached board photos includes connector labels
  such as `P4`, `P6`, `P9`, `P10`, `HDMI`, and `5V IN`.
- No clearly legible PCB revision string is recoverable from the provided photo
  resolution; revision remains visually inspected but not transcribed exactly.
- Available power input options recorded from known BBB hardware family:
  USB and 5V barrel input are expected; operator confirms first power-on used
  USB power.

## Image Provenance

- [x] Official image source URL recorded
- [x] Image filename/version recorded
- [x] Published checksum recorded
- [x] Checksum verification result retained
- [x] Target removable device mapping retained
- [x] Exact imaging command retained
- [x] Post-write partition or filesystem observation retained
- [x] T1 decision recorded: PASS / HOLD / FAIL

Evidence Notes
- Attached screenshot indicates official image source page with image and
  `sha256` download controls for `BeagleBone Black Debian 13.4 2026-03-17 IoT
  (v6.19.x)`.
- Attached screenshot indicates image family: `BeagleBone Black Debian 13.4
  2026-03-17 IoT (v6.19.x)`.
- Attached file list shows image filename:
  `am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz`.
- Attached checksum text:
  `ee385787ec5d13df239caaea58344a677b6f5ad59ffddf2a3ce7b4e0f541df93`.
- Local checksum sidecar appears present in screenshot as
  `am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz.sha256sum`.
- Exact source URL recorded:
  `https://files.beagle.cc/file/beagleboard-public-2021/images/am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz`
- Operator reports the board was flashed from the verified image.
- Host checksum output retained:
  `ee385787ec5d13df239caaea58344a677b6f5ad59ffddf2a3ce7b4e0f541df93  am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz`
- Sidecar checksum file contents retained and matched the host checksum output.
- A later command bundle intended for T2 was executed on the Omarchy host
  instead of the BBB target and is not accepted as target evidence.
- Retained host-side USB gadget interface evidence:
  `enp0s29u1u3` on `192.168.7.1/24`, matching the BBB-side `usb0`
  connectivity at `192.168.7.2/24`.
- Host removable-device mapping retained before write:
  `mmcblk0 59.7G TRAN=mmc RM=0` with partitions `mmcblk0p1 512M`,
  `mmcblk0p2 59.2G`.
- Host media unmount step retained:
  `sudo umount /dev/mmcblk0p1` and `sudo umount /dev/mmcblk0p2`.
- Flash transcript retained:
  `xzcat am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz | sudo dd of=/dev/mmcblk0 bs=4M conv=fsync status=progress`
  followed by `sync`.
- Flash write completion retained:
  `3774873600 bytes (3.8 GB, 3.5 GiB) copied, 150.536 s, 25.1 MB/s`.
- Post-write partition/filesystem observation retained:
  `mmcblk0p1 36M vfat BOOT`, `mmcblk0p2 512M swap`, `mmcblk0p3 3G ext4 rootfs`.
- Post-write partition table retained from `fdisk -l /dev/mmcblk0`.
- Subsequent host `lsblk` with `mmcblk0` absent is consistent with card removal
  after imaging.
- T1 decision: PASS with retained image provenance, verified checksum, actual
  write target `/dev/mmcblk0`, and post-write media observation.

Planned Host Command
```bash
sha256sum am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz
lsblk -o NAME,SIZE,MODEL,SERIAL,TRAN,RM,MOUNTPOINT
xzcat am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb.img.xz | sudo dd of=/dev/sdX bs=4M conv=fsync status=progress
sync
lsblk -o NAME,SIZE,FSTYPE,LABEL,MOUNTPOINT /dev/sdX
```

Actual Host Command Retained
```bash
xzcat am335x-debian-13.4-base-v6.19-armhf-2026-03-17-4gb (2).img.xz | sudo dd of=/dev/sdX bs=4M conv=fsync status=progress
sync
```

## First Boot

- [x] Ethernet remained disconnected
- [x] Operator access path recorded
- [x] Boot log retained
- [x] Console transcript retained
- [x] Kernel version recorded
- [x] Root filesystem or boot source recorded
- [x] Boot confirmed from microSD (not eMMC)
- [x] Block device mapping retained (rootfs source)
- [x] Observed interfaces retained
- [x] T2 decision recorded: PASS / HOLD / FAIL

Evidence Notes
- Operator reports first shell access via USB SSH at `192.168.7.2`.
- This is a local-only fallback operator path and must be recorded as such if
  retained; preferred target remains UART-only.
- Operator access path observed from retained transcript:
  `ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null debian@192.168.7.2`
- Target login banner retained:
  `Debian GNU/Linux 13`
- Image banner retained:
  `BeagleBoard.org Debian Trixie Base Image 2026-03-17`
- Support banner retained:
  `Support: https://bbb.io/debian`
- Credential banner retained:
  `default username is [debian] with a one time password of [temppwd]`
- Login banner retained:
  `Debian GNU/Linux comes with ABSOLUTELY NO WARRANTY, to the extent permitted by applicable law.`
- Session evidence retained:
  `Last login: Tue Mar 17 18:34:39 2026 from 192.168.7.1`
- Interactive shell was stable enough to launch `htop` and exit cleanly.
- A later evidence bundle containing `uname -a`, `/etc/os-release`, `ip`, and
  `lsblk` output was captured on the Omarchy host (`Arch Linux`, host
  `lappy`) and is excluded from T2 acceptance.
- Valid target kernel output retained:
  `Linux BeagleBone 6.19.6-bone11 #1 SMP PREEMPT Fri Mar  6 09:13:48 UTC 2026 armv7l GNU/Linux`
- Valid target OS release retained:
  `PRETTY_NAME="Debian GNU/Linux 13 (trixie)"`, `DEBIAN_VERSION_FULL=13.4`
- Root filesystem source retained from target:
  `/dev/mmcblk0p3`
- Boot-source inference: on BBB this rootfs device is consistent with microSD
  boot, not eMMC.
- Valid target interface snapshot retained:
  `eth0 DOWN <NO-CARRIER,...>`, `usb0 UP 192.168.7.2/24`, loopback present.
- Host-side USB link evidence retained:
  `enp0s29u1u3 UP 192.168.7.1/24`.
- Explicit operator confirmation retained:
  `Ethernet disconnected: yes`
- Explicit operator boundary declaration retained:
  `Operator Boundary: local-only (justified)`
- Target `lsblk -o NAME,MOUNTPOINT,SOURCE` failed because this `lsblk` build
  does not support the `SOURCE` column; `findmnt -no SOURCE /` was used as the
  rootfs-source evidence instead.
- Preferred operator boundary target remains UART-only.
- T2 decision: PASS for isolated first boot with a documented local-only
  boundary. This does not imply trusted hardware.

## Services / Interfaces

- [ ] Enabled-unit snapshot retained
- [ ] `ip link` snapshot retained
- [ ] `ip addr` snapshot retained
- [ ] `ss -tulpen` snapshot retained before reduction
- [ ] Listener snapshot retained after reduction
- [x] Credential-change result recorded (changed / not applicable)
- [ ] Persistent config changes recorded
- [x] Operator boundary declared in required format
- [ ] T3 decision recorded: PASS / HOLD / FAIL

Evidence Notes
- Default one-time password banner was observed at login. Credential change is
  no longer pending; operator reports the credential was changed.
- Current observed local-only path is USB gadget networking over `usb0`; this
  must be declared explicitly if retained.
- Credential-expiry enforcement transcript retained:
  `You are required to change your password immediately (administrator enforced).`
- Password change completion retained:
  `passwd: password updated successfully`

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

Evidence Notes
- No T4 evidence captured yet.

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

Current State
- T0: PASS
- T1: PASS
- T2: PASS
- T3-T5: not started

Open Unknowns
- Visible board revision markings
- Target microSD device path on the Omarchy laptop
- UART adapter/path details for isolated first boot
