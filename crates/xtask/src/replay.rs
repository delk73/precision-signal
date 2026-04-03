pub(crate) fn usage() -> String {
    "usage: cargo xtask usb doctor --board <id-or-path>\n       cargo xtask usb flash --board <id-or-path> --elf <path/to/fw.elf> [--log <path>] [--execute]\n       cargo xtask usb debug --board <id-or-path> [--execute]\n       optional flags: --json --verbose"
        .to_string()
}
