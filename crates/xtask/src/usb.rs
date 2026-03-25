use crate::board::{BoardDescriptor, PreferredBackend};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Pass,
    Warn,
    Fail,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Warn => "WARN",
            Status::Fail => "FAIL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub status: Status,
    pub check_id: String,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub schema_version: String,
    pub overall: Status,
    pub checks: Vec<Check>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashPlan {
    pub steps: Vec<CommandSpec>,
    pub cleanup_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugPlan {
    pub spawn: CommandSpec,
    pub attach_port: u16,
    pub attach_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub command: String,
    pub args: Vec<String>,
}

pub fn empty_report() -> Report {
    Report {
        schema_version: "usb-report.v1".to_string(),
        overall: Status::Pass,
        checks: Vec::new(),
        events: Vec::new(),
    }
}

pub fn single_check_report(
    status: Status,
    check_id: &str,
    message: &str,
    hint: Option<&str>,
) -> Report {
    let mut report = empty_report();
    report.checks.push(Check {
        status,
        check_id: check_id.to_string(),
        message: message.to_string(),
        hint: hint.map(ToString::to_string),
    });
    finalize_report(report)
}

pub fn doctor_report(board: &BoardDescriptor, verbose: bool) -> Report {
    let mut checks = Vec::new();
    checks.push(Check {
        status: Status::Pass,
        check_id: "board.required_fields".to_string(),
        message: "required fields present".to_string(),
        hint: None,
    });
    checks.push(Check {
        status: Status::Pass,
        check_id: "board.preferred_backend".to_string(),
        message: format!(
            "preferred backend is {}",
            backend_name(board.preferred_backend)
        ),
        hint: None,
    });

    let ports = board.ports.as_ref();
    match board.preferred_backend {
        PreferredBackend::Stlink => {
            let stutil = ports.and_then(|p| p.stutil).unwrap_or(4242);
            checks.push(Check {
                status: Status::Pass,
                check_id: "usb.stlink.port_stutil".to_string(),
                message: format!("st-util port is {stutil}"),
                hint: None,
            });
        }
        PreferredBackend::Openocd => {
            let gdb = ports.and_then(|p| p.gdb).unwrap_or(3333);
            let tcl = ports.and_then(|p| p.openocd_tcl).unwrap_or(6666);
            let telnet = ports.and_then(|p| p.openocd_telnet).unwrap_or(4444);
            checks.push(Check {
                status: Status::Pass,
                check_id: "usb.openocd.port_gdb".to_string(),
                message: format!("OpenOCD GDB port is {gdb}"),
                hint: None,
            });
            checks.push(Check {
                status: Status::Pass,
                check_id: "usb.openocd.port_tcl".to_string(),
                message: format!("OpenOCD TCL port is {tcl}"),
                hint: None,
            });
            checks.push(Check {
                status: Status::Pass,
                check_id: "usb.openocd.port_telnet".to_string(),
                message: format!("OpenOCD telnet port is {telnet}"),
                hint: None,
            });
            if board.openocd_interface_cfg.is_none() || board.openocd_target_cfg.is_none() {
                checks.push(Check {
                    status: Status::Warn,
                    check_id: "usb.openocd.cfg_files".to_string(),
                    message: "openocd_*_cfg values are not both set".to_string(),
                    hint: Some("set openocd_interface_cfg and openocd_target_cfg".to_string()),
                });
            }
        }
    }
    let events = if verbose {
        vec![Event {
            kind: "doctor_mode".to_string(),
            text: "doctor runs read-only checks only".to_string(),
        }]
    } else {
        Vec::new()
    };
    finalize_report(Report {
        schema_version: "usb-report.v1".to_string(),
        overall: Status::Pass,
        checks,
        events,
    })
}

pub fn flash_plan(board: &BoardDescriptor, elf_path: &str, board_name: &str) -> FlashPlan {
    match board.preferred_backend {
        PreferredBackend::Stlink => {
            let temp_bin = Path::new("target")
                .join("xtask")
                .join("usb")
                .join(format!("{board_name}.bin"));
            FlashPlan {
                steps: vec![
                    CommandSpec {
                        command: "rust-objcopy".to_string(),
                        args: vec![
                            "-O".to_string(),
                            "binary".to_string(),
                            elf_path.to_string(),
                            temp_bin.display().to_string(),
                        ],
                    },
                    CommandSpec {
                        command: "st-flash".to_string(),
                        args: vec![
                            "write".to_string(),
                            temp_bin.display().to_string(),
                            format!("0x{:08X}", board.flash_base),
                        ],
                    },
                ],
                cleanup_paths: vec![temp_bin],
            }
        }
        PreferredBackend::Openocd => {
            let interface = board
                .openocd_interface_cfg
                .as_deref()
                .unwrap_or("interface/stlink.cfg");
            let target = board
                .openocd_target_cfg
                .as_deref()
                .unwrap_or("target/stm32f4x.cfg");
            let ports = board.ports.as_ref();
            let gdb = ports.and_then(|p| p.gdb).unwrap_or(3333);
            let tcl = ports.and_then(|p| p.openocd_tcl).unwrap_or(6666);
            let telnet = ports.and_then(|p| p.openocd_telnet).unwrap_or(4444);
            FlashPlan {
                steps: vec![CommandSpec {
                    command: "openocd".to_string(),
                    args: vec![
                        "-f".to_string(),
                        interface.to_string(),
                        "-f".to_string(),
                        target.to_string(),
                        "-c".to_string(),
                        format!(
                            "gdb_port {gdb}; tcl_port {tcl}; telnet_port {telnet}; program {elf_path} verify reset exit"
                        ),
                    ],
                }],
                cleanup_paths: Vec::new(),
            }
        }
    }
}

pub fn debug_plan(board: &BoardDescriptor) -> DebugPlan {
    let ports = board.ports.as_ref();
    match board.preferred_backend {
        PreferredBackend::Stlink => {
            let stutil = ports.and_then(|p| p.stutil).unwrap_or(4242);
            DebugPlan {
                spawn: CommandSpec {
                    command: "st-util".to_string(),
                    args: vec!["-p".to_string(), stutil.to_string()],
                },
                attach_port: stutil,
                attach_hint: format!("start `st-util -p {stutil}` and attach GDB to :{stutil}"),
            }
        }
        PreferredBackend::Openocd => {
            let gdb = ports.and_then(|p| p.gdb).unwrap_or(3333);
            let tcl = ports.and_then(|p| p.openocd_tcl).unwrap_or(6666);
            let telnet = ports.and_then(|p| p.openocd_telnet).unwrap_or(4444);
            let interface = board
                .openocd_interface_cfg
                .as_deref()
                .unwrap_or("interface/stlink.cfg");
            let target = board
                .openocd_target_cfg
                .as_deref()
                .unwrap_or("target/stm32f4x.cfg");
            DebugPlan {
                spawn: CommandSpec {
                    command: "openocd".to_string(),
                    args: vec![
                        "-f".to_string(),
                        interface.to_string(),
                        "-f".to_string(),
                        target.to_string(),
                        "-c".to_string(),
                        format!("gdb_port {gdb}; tcl_port {tcl}; telnet_port {telnet}; init; reset halt"),
                    ],
                },
                attach_port: gdb,
                attach_hint: format!("attach GDB to :{gdb}"),
            }
        }
    }
}

pub fn finalize_report(mut report: Report) -> Report {
    report.checks.sort_by(|a, b| {
        a.check_id
            .cmp(&b.check_id)
            .then(a.status.cmp(&b.status))
            .then(a.message.cmp(&b.message))
            .then(a.hint.cmp(&b.hint))
    });
    report.overall = summarize_overall(&report.checks);
    report
}

pub fn plan_report(action: &str, commands: &[CommandSpec], verbose: bool) -> Report {
    let mut report = empty_report();
    report.checks.push(Check {
        status: Status::Pass,
        check_id: format!("usb.{action}.plan"),
        message: "command line assembled".to_string(),
        hint: None,
    });
    if verbose {
        for command in commands {
            report.events.push(Event {
                kind: "spawn_command_line".to_string(),
                text: format_command_line(command),
            });
        }
    }
    finalize_report(report)
}

pub fn format_command_line(command: &CommandSpec) -> String {
    let mut parts = Vec::with_capacity(1 + command.args.len());
    parts.push(quote_arg(&command.command));
    for arg in &command.args {
        parts.push(quote_arg(arg));
    }
    parts.join(" ")
}

fn quote_arg(input: &str) -> String {
    let needs_quotes = input
        .chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '\\');
    if !needs_quotes {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len() + 2);
    out.push('"');
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

pub fn summarize_overall(checks: &[Check]) -> Status {
    if checks.iter().any(|c| c.status == Status::Fail) {
        Status::Fail
    } else if checks.iter().any(|c| c.status == Status::Warn) {
        Status::Warn
    } else {
        Status::Pass
    }
}

fn backend_name(backend: PreferredBackend) -> &'static str {
    match backend {
        PreferredBackend::Stlink => "stlink",
        PreferredBackend::Openocd => "openocd",
    }
}

#[cfg(test)]
mod tests {
    use crate::board::parse_board_descriptor;

    use super::{doctor_report, flash_plan, format_command_line, plan_report};

    #[test]
    fn stlink_does_not_touch_gdb_port() {
        let board = parse_board_descriptor(
            r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "stlink"

[ports]
gdb = 0
stutil = 4242
"#,
        )
        .expect("board should parse");

        let report = doctor_report(&board, false);
        assert!(
            !report.checks.iter().any(|c| c.check_id.contains("gdb")),
            "stlink checks must not reference gdb"
        );
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "usb.stlink.port_stutil"),
            "stlink path should validate stutil"
        );
    }

    #[test]
    fn flash_plan_stlink_has_objcopy_and_st_flash_with_target_temp_bin() {
        let board = parse_board_descriptor(
            r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "stlink"
"#,
        )
        .expect("board should parse");
        let plan = flash_plan(&board, "target/fw.elf", "myboard");
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].command, "rust-objcopy");
        assert_eq!(
            plan.steps[0].args,
            vec![
                "-O".to_string(),
                "binary".to_string(),
                "target/fw.elf".to_string(),
                "target/xtask/usb/myboard.bin".to_string()
            ]
        );
        assert_eq!(plan.steps[1].command, "st-flash");
        assert_eq!(
            plan.steps[1].args,
            vec![
                "write".to_string(),
                "target/xtask/usb/myboard.bin".to_string(),
                "0x08000000".to_string()
            ]
        );
        assert_eq!(plan.cleanup_paths.len(), 1);
        assert_eq!(
            plan.cleanup_paths[0].display().to_string(),
            "target/xtask/usb/myboard.bin"
        );
    }

    #[test]
    fn flash_plan_openocd_is_elf_only() {
        let board = parse_board_descriptor(
            r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "openocd"
"#,
        )
        .expect("board should parse");
        let plan = flash_plan(&board, "target/fw.elf", "ignored");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].command, "openocd");
        let line = plan.steps[0].args.join(" ");
        assert!(line.contains("program target/fw.elf verify reset exit"));
        assert!(plan.cleanup_paths.is_empty());
    }

    #[test]
    fn events_order_preserved_for_flash_plan() {
        let board = parse_board_descriptor(
            r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "stlink"
"#,
        )
        .expect("board should parse");
        let plan = flash_plan(&board, "target/fw.elf", "myboard");
        let report = plan_report("flash", &plan.steps, true);
        assert_eq!(report.events.len(), 2);
        assert!(report.events[0].text.starts_with("rust-objcopy "));
        assert!(report.events[1].text.starts_with("st-flash "));
    }

    #[test]
    fn render_command_line_quotes_paths_with_spaces() {
        let command = super::CommandSpec {
            command: "rust-objcopy".to_string(),
            args: vec![
                "-O".to_string(),
                "binary".to_string(),
                "target/firmware with space.elf".to_string(),
                "target/xtask/usb/my board.bin".to_string(),
            ],
        };
        let rendered = format_command_line(&command);
        assert_eq!(
            rendered,
            "rust-objcopy -O binary \"target/firmware with space.elf\" \"target/xtask/usb/my board.bin\""
        );
    }
}
