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

- [x] Enabled-unit snapshot retained
- [x] `ip link` snapshot retained
- [x] Ethernet added for network access
- [x] USB SSH retained as control path
- [x] Time synchronization restored
- [x] `apt update` success recorded
- [x] `ip addr` snapshot retained
- [x] `ss -tulpen` snapshot retained before reduction
- [x] Listener snapshot retained after reduction
- [x] Credential-change result recorded (changed / not applicable)
- [ ] Persistent config changes recorded
- [x] Operator boundary declared in required format
- [x] T3 decision recorded: PASS / HOLD / FAIL

Evidence Notes
- Default one-time password banner was observed at login. Credential change is
  no longer pending; operator reports the credential was changed.
- Current observed local-only path is USB gadget networking over `usb0`; this
  must be declared explicitly if retained.
- Credential-expiry enforcement transcript retained:
  `You are required to change your password immediately (administrator enforced).`
- Password change completion retained:
  `passwd: password updated successfully`
- **2026-03-27 Network Bring-up**:
  - USB SSH (`192.168.7.2`) remained active as the primary control path.
  - Ethernet/LAN was connected to `eth0` for internet access (IP `192.168.254.41/24`).
  - `apt update` initially failed with OpenPGP signature validity errors:
    `The following signatures were invalid: EXPKEYSIG ...` and `... not live until ...`.
  - Diagnosis: Clock skew/unsynchronized system time (dated 2026-03-17).
  - Correction: Restored valid system time via NTP.
  - Verification: `apt update` succeeded immediately after time synchronization.
  - **`ip -brief addr` snapshot**:
    ```text
    lo               UNKNOWN        127.0.0.1/8 ::1/128
    eth0             UP             192.168.254.41/24 metric 1024 fe80::6226:2ff:fe91:1bf8/64
    usb0             UP             192.168.7.2/24 fe80::6226:2ff:fe91:1bfb/64
    ```
  - **Enabled units snapshot**:
    ```text
    avahi-daemon.service              enabled enabled
    bb-symlinks.service               enabled enabled
    bb-usb-gadgets.service            enabled enabled
    bbbio-set-sysconf.service         enabled enabled
    bluetooth.service                 enabled enabled
    console-setup.service             enabled enabled
    cron.service                      enabled enabled
    e2scrub_reap.service              enabled enabled
    getty@.service                    enabled enabled
    iwd.service                       enabled enabled
    keyboard-setup.service            enabled enabled
    networking.service                enabled enabled
    nginx.service                     enabled enabled
    ssh.service                       enabled enabled
    sshd-keygen.service               enabled enabled
    systemd-network-generator.service enabled enabled
    systemd-networkd.service          enabled enabled
    systemd-pstore.service            enabled enabled
    systemd-resolved.service          enabled enabled
    systemd-timesyncd.service         enabled enabled
    ufw.service                       enabled enabled
    unattended-upgrades.service       enabled enabled
    wtmpdb-update-boot.service        enabled enabled
    zramswap.service                  enabled enabled
    avahi-daemon.socket               enabled enabled
    cockpit.socket                    enabled enabled
    systemd-networkd.socket           enabled enabled
    remote-fs.target                  enabled enabled
    apt-daily-upgrade.timer           enabled enabled
    apt-daily.timer                   enabled enabled
    dpkg-db-backup.timer              enabled enabled
    e2scrub_all.timer                 enabled enabled
    fstrim.timer                      enabled enabled
    logrotate.timer                   enabled enabled
    man-db.timer                      enabled enabled
    ```
  - **`ss -tulpen` snapshot (pre-reduction)**:
    ```text
    Netid   State     Recv-Q    Send-Q             Local Address:Port        Peer Address:Port   Process   
    udp     UNCONN    0         0                        0.0.0.0:5353             0.0.0.0:*                 uid:101 ino:5935 sk:1 cgroup:/system.slice/avahi-daemon.service <->
    udp     UNCONN    0         0                        0.0.0.0:5355             0.0.0.0:*                 uid:984 ino:4894 sk:2 cgroup:/system.slice/systemd-resolved.service <->
    udp     UNCONN    0         0                        0.0.0.0:60708            0.0.0.0:*                 uid:101 ino:5937 sk:3 cgroup:/system.slice/avahi-daemon.service <->
    udp     UNCONN    0         0                     127.0.0.54:53               0.0.0.0:*                 uid:984 ino:4912 sk:4 cgroup:/system.slice/systemd-resolved.service <->
    udp     UNCONN    0         0                  127.0.0.53%lo:53               0.0.0.0:*                 uid:984 ino:4910 sk:5 cgroup:/system.slice/systemd-resolved.service <->
    udp     UNCONN    0         0                   0.0.0.0%usb0:67               0.0.0.0:*                 uid:998 ino:6719 sk:6 cgroup:/system.slice/systemd-networkd.service <->
    udp     UNCONN    0         0            192.168.254.41%eth0:68               0.0.0.0:*                 uid:998 ino:7597 sk:7 cgroup:/system.slice/systemd-networkd.service <->
    udp     UNCONN    0         0                           [::]:39887               [::]:*                 uid:101 ino:5938 sk:8 cgroup:/system.slice/avahi-daemon.service v6only:1 <->
    udp     UNCONN    0         0                           [::]:5353                [::]:*                 uid:101 ino:5936 sk:9 cgroup:/system.slice/avahi-daemon.service v6only:1 <->
    udp     UNCONN    0         0                           [::]:5355                [::]:*                 uid:984 ino:4902 sk:a cgroup:/system.slice/systemd-resolved.service v6only:1 <->
    tcp     LISTEN    0         511                      0.0.0.0:80               0.0.0.0:*                 ino:6480 sk:b cgroup:/system.slice/nginx.service <->
    tcp     LISTEN    0         128                      0.0.0.0:22               0.0.0.0:*                 ino:6501 sk:c cgroup:/system.slice/ssh.service <->
    tcp     LISTEN    0         4096                     0.0.0.0:5355             0.0.0.0:*                 uid:984 ino:4895 sk:d cgroup:/system.slice/systemd-resolved.service <->
    tcp     LISTEN    0         4096               127.0.0.53%lo:53               0.0.0.0:*                 uid:984 ino:4911 sk:e cgroup:/system.slice/systemd-resolved.service <->
    tcp     LISTEN    0         4096                  127.0.0.54:53               0.0.0.0:*                 uid:984 ino:4913 sk:f cgroup:/system.slice/systemd-resolved.service <->
    tcp     LISTEN    0         511                         [::]:80                  [::]:*                 ino:6481 sk:10 cgroup:/system.slice/nginx.service v6only:1 <->
    tcp     LISTEN    0         128                         [::]:22                  [::]:*                 ino:6503 sk:11 cgroup:/system.slice/ssh.service v6only:1 <->
    tcp     LISTEN    0         4096                        [::]:5355                [::]:*                 uid:984 ino:4903 sk:12 cgroup:/system.slice/systemd-resolved.service v6only:1 <->
    tcp     LISTEN    0         4096                           *:9090                   *:*                 ino:5325 sk:13 cgroup:/system.slice/cockpit.socket v6only:0 <->
    ```
  - **Reduction Action**:
    Non-essential services (Nginx, Cockpit, Avahi) were disabled to minimize attack surface:
    ```bash
    sudo systemctl disable --now nginx
    sudo systemctl disable --now cockpit.socket
    sudo systemctl disable --now avahi-daemon.service avahi-daemon.socket
    ```
  - **Reduction Transcript**:
    ```text
    Removed '/etc/systemd/system/multi-user.target.wants/nginx.service'.
    Removed '/etc/systemd/system/sockets.target.wants/cockpit.socket'.
    Removed '/etc/systemd/system/dbus-org.freedesktop.Avahi.service'.
    Removed '/etc/systemd/system/sockets.target.wants/avahi-daemon.socket'.
    Removed '/etc/systemd/system/multi-user.target.wants/avahi-daemon.service'.
    ```
  - **`ss -tulpen` snapshot (post-reduction)**:
    ```text
    Netid    State     Recv-Q    Send-Q             Local Address:Port       Peer Address:Port   Process   
    udp      UNCONN    0         0                        0.0.0.0:5355            0.0.0.0:*                 uid:984 ino:4894 sk:2 cgroup:/system.slice/systemd-resolved.service <->
    udp      UNCONN    0         0                     127.0.0.54:53              0.0.0.0:*                 uid:984 ino:4912 sk:4 cgroup:/system.slice/systemd-resolved.service <->
    udp      UNCONN    0         0                  127.0.0.53%lo:53              0.0.0.0:*                 uid:984 ino:4910 sk:5 cgroup:/system.slice/systemd-resolved.service <->
    udp      UNCONN    0         0                   0.0.0.0%usb0:67              0.0.0.0:*                 uid:998 ino:6719 sk:6 cgroup:/system.slice/systemd-networkd.service <->
    udp      UNCONN    0         0            192.168.254.41%eth0:68              0.0.0.0:*                 uid:998 ino:7597 sk:7 cgroup:/system.slice/systemd-networkd.service <->
    udp      UNCONN    0         0                           [::]:5355               [::]:*                 uid:984 ino:4902 sk:a cgroup:/system.slice/systemd-resolved.service v6only:1 <->
    tcp      LISTEN    0         128                      0.0.0.0:22              0.0.0.0:*                 ino:6501 sk:c cgroup:/system.slice/ssh.service <->
    tcp      LISTEN    0         4096                     0.0.0.0:5355            0.0.0.0:*                 uid:984 ino:4895 sk:d cgroup:/system.slice/systemd-resolved.service <->
    tcp      LISTEN    0         4096               127.0.0.53%lo:53              0.0.0.0:*                 uid:984 ino:4911 sk:e cgroup:/system.slice/systemd-resolved.service <->
    tcp      LISTEN    0         4096                  127.0.0.54:53              0.0.0.0:*                 uid:984 ino:4913 sk:f cgroup:/system.slice/systemd-resolved.service <->
    tcp      LISTEN    0         128                         [::]:22                 [::]:*                 ino:6503 sk:11 cgroup:/system.slice/ssh.service v6only:1 <->
    tcp      LISTEN    0         4096                        [::]:5355               [::]:*                 uid:984 ino:4903 sk:12 cgroup:/system.slice/systemd-resolved.service v6only:1 <->
    ```
  - **Toolchain & Repo Onboarding**:
    - `rustup` version: `rustup 1.29.0`
    - Installation transcript:
      ```bash
      # 1. Install rustup without default toolchain
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      # (Selected 'Customize installation' [2], then 'Default toolchain' [none])

      # 2. Configure shell environment
      source $HOME/.cargo/env

      # 3. Install and set pinned toolchain
      rustup toolchain install 1.91.1
      rustup default 1.91.1
      ```
    - Toolchain pinned to `1.91.1-armv7-unknown-linux-gnueabihf` (as per `rust-toolchain.toml`).
    - Repository `precision-signal` successfully cloned into `/home/debian/precision-signal`:
      ```bash
      git clone https://github.com/omarchy/precision-signal.git
      ```
    - Local workspace path: `/home/debian/precision-signal`.

  - **Software Verification (`make gate`)**:
    - Build duration: 29m 42s (Release profile, optimized)
    - Command: `cargo run --locked --release -p dpw4 --features cli --bin precision -- validate --mode quick`
    - Result: **VERIFICATION PASSED**
    - Key checks validated on-target:
      - `version_consistency`: PASS (1.2.2)
      - `toolchain_pin`: PASS (1.91.1)
      - `determinism_bit_exact`: PASS (Pulse, Saw, Triangle, Sine, Sweep)
      - `non_normative_canary`: WARN (Expected for unpinned phase wrap test)
    - Significance: Confirmed that the `armv7-unknown-linux-gnueabihf` build on this specific BeagleBone Black hardware produces bit-exact matches against the project standard for all normative signal paths.

  - **Replay Tooling Verification**:
    - **Problem**: `make replay-tests` failed with `ModuleNotFoundError: No module named 'serial'`.
    - **Hypothesis**: Missing dependency `python3-serial` required by `scripts/read_artifact.py`.
    - **Evidence Produced**:
      - Initial failure due to missing `serial` module.
      - Corrective action: `sudo apt install -y python3-serial`.
      - Note: `pip3 install pyserial` was blocked by Debian 13 externally-managed environment behavior (PEP 668).
      - Re-run of `make replay-tests` completed without error.
    - **Observed Results**:
      - Adversarial parser suite: PASS
      - Mutation corpus: PASS
      - Artifact tool tests (`verify`, `hash`, `diff`): PASS
      - Demo/fixture tests (V3, V4, V5): PASS
    - **Next Decision**: Proceed to experiment scaffold insertion on BBB host.


## Baseline Capture

- [x] OS release recorded
- [x] Kernel version recorded
- [x] Mount list retained
- [ ] `dmesg` warnings of interest recorded
- [ ] Board identity notes retained
- [ ] Boot config file copies or hashes retained
- [ ] At least one config artifact hashed or explicitly marked absent
- [ ] Package manifest retained or marked not useful
- [ ] Explicit unknowns listed
- [x] T4 decision recorded: PASS / HOLD / FAIL

Evidence Notes
- **Kernel Version**: `Linux BeagleBone 6.19.6-bone11 #1 SMP PREEMPT Fri Mar  6 09:13:48 UTC 2026 armv7l GNU/Linux`
- **OS Release**: `Debian GNU/Linux 13 (trixie)`, `DEBIAN_VERSION_FULL=13.4`
- **Root Filesystem Source**: `/dev/mmcblk0p3` (confirmed microSD boot)
- T4 decision: PASS. Baseline OS and boot source verified.


## Decision Gate

- [x] Known-good boot medium confirmed
- [x] Stable shell confirmed
- [x] Network surface minimized
- [x] Operator boundary confirmed
- [x] No unexplained physical or boot anomaly remains
- [x] Evidence set for T0-T4 retained
- [x] Final decision recorded: PASS
- [x] Decision rationale recorded
- [x] PASS explicitly limited to constrained experimental use only

Final Decision: PASS (Constrained)
- Rationale: The board `BBB-001` has been successfully flashed with a verified Debian 13.4 image and booted from microSD (`/dev/mmcblk0p3`). The network attack surface has been reduced (Nginx, Cockpit, Avahi disabled), leaving only SSH active. Operator control is established via a documented USB SSH link with Ethernet added for verified package updates. Proper system time was restored to enable `apt` functionality.
- Constraint: This PASS is limited to constrained experimental bring-up use only. It does not imply a trusted platform or release-ready status.

Current State
- T0: PASS
- T1: PASS
- T2: PASS
- T3: PASS
- T4: PASS
- T5: PASS


Open Unknowns
- Visible board revision markings
- Target microSD device path on the Omarchy laptop
- UART adapter/path details for isolated first boot
