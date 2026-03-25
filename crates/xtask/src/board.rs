use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreferredBackend {
    Stlink,
    Openocd,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ports {
    pub gdb: Option<u16>,
    pub stutil: Option<u16>,
    pub openocd_tcl: Option<u16>,
    pub openocd_telnet: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardDescriptor {
    pub chip: String,
    pub flash_base: u32,
    pub preferred_backend: PreferredBackend,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub openocd_interface_cfg: Option<String>,
    pub openocd_target_cfg: Option<String>,
    pub ports: Option<Ports>,
}

pub fn parse_board_descriptor(raw: &str) -> Result<BoardDescriptor, String> {
    toml::from_str::<BoardDescriptor>(raw).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_board_descriptor;

    #[test]
    fn rejects_unknown_top_level_key() {
        let input = r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "stlink"
mystery = "nope"
"#;
        let err = parse_board_descriptor(input).expect_err("unknown keys must fail");
        assert!(err.contains("unknown field"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_unknown_table() {
        let input = r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "openocd"

[unexpected]
thing = 1
"#;
        let err = parse_board_descriptor(input).expect_err("unknown tables must fail");
        assert!(err.contains("unknown field"), "unexpected error: {err}");
    }

    #[test]
    fn enforces_required_fields() {
        let input = r#"
chip = "STM32F446"
preferred_backend = "stlink"
"#;
        let err = parse_board_descriptor(input).expect_err("missing flash_base must fail");
        assert!(err.contains("missing field"), "unexpected error: {err}");
    }
}
