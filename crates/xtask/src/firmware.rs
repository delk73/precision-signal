use crate::release::BOARD_FILE_MISSING_HINT;
use crate::util::yyyymmdd_utc;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use xtask::board::{parse_board_descriptor, BoardDescriptor};
use xtask::usb::{
    empty_report, finalize_report, format_command_line, single_check_report, Check, CommandSpec,
    Event, Report, Status,
};

pub(crate) fn stlink_image_compat_plan(
    board: &BoardDescriptor,
    image_path: &str,
) -> xtask::usb::FlashPlan {
    xtask::usb::FlashPlan {
        steps: vec![CommandSpec {
            command: "st-flash".to_string(),
            args: vec![
                "write".to_string(),
                image_path.to_string(),
                format!("0x{:08X}", board.flash_base),
            ],
        }],
        cleanup_paths: Vec::new(),
    }
}

pub(crate) fn run_flash_execute(
    repo_root: &Path,
    plan: &xtask::usb::FlashPlan,
    log_path: &Path,
    verbose: bool,
) -> Report {
    let mut report = empty_report();
    let mut log_lines = Vec::new();
    let mut failing_check_id: Option<String> = None;
    log_lines.push(format!("log_path: {}", log_path.display()));
    let unique_tools: BTreeSet<&str> = plan
        .steps
        .iter()
        .map(|step| step.command.as_str())
        .collect();
    for tool in unique_tools {
        if !tool_present(tool) {
            report.checks.push(Check {
                status: Status::Warn,
                check_id: format!(
                    "usb.flash.tool_missing.{}",
                    sanitize_check_id_component(tool)
                ),
                message: format!("tool not found in PATH: {tool}"),
                hint: Some("install required flashing tool or adjust PATH".to_string()),
            });
        }
    }
    if needs_usb_tempdir(plan) {
        let temp_dir = repo_root.join("target").join("xtask").join("usb");
        if let Err(err) = fs::create_dir_all(&temp_dir) {
            report.checks.push(Check {
                status: Status::Fail,
                check_id: "usb.flash.tempdir.create".to_string(),
                message: format!("failed to create temp dir '{}': {err}", temp_dir.display()),
                hint: Some("verify write permissions under target/".to_string()),
            });
            if let Err(write_err) = write_log(log_path, &log_lines.join("\n")) {
                report.checks.push(Check {
                    status: Status::Fail,
                    check_id: "usb.flash.runlog.write".to_string(),
                    message: format!("failed to write runlog: {write_err}"),
                    hint: Some("check --log path permissions".to_string()),
                });
            }
            return finalize_report(report);
        }
    }
    for (step_index, step) in plan.steps.iter().enumerate() {
        let step_check_id = format!(
            "usb.flash.execute_step.{step_index:02}.{}",
            sanitize_check_id_component(&step.command)
        );
        log_lines.push(format!("command: {}", format_command_line(step)));
        if verbose {
            report.events.push(Event {
                kind: "spawn_command_line".to_string(),
                text: format_command_line(step),
            });
        }
        match run_capture(step) {
            Ok(output) => {
                log_lines.push(format!("status: {}", output.status_code));
                log_lines.push(format!("stdout_bytes: {}", output.stdout_bytes));
                log_lines.push(format!("stderr_bytes: {}", output.stderr_bytes));
                log_lines.push("stdout:".to_string());
                log_lines.extend(output.stdout.lines().map(ToString::to_string));
                log_lines.push("stderr:".to_string());
                log_lines.extend(output.stderr.lines().map(ToString::to_string));
                if output.status_code == 0 {
                    report.checks.push(Check {
                        status: Status::Pass,
                        check_id: step_check_id.clone(),
                        message: format!("completed: {}", step.command),
                        hint: None,
                    });
                } else {
                    report.checks.push(Check {
                        status: Status::Fail,
                        check_id: step_check_id.clone(),
                        message: format!(
                            "non-zero exit from {} (exit={})",
                            step.command, output.status_code
                        ),
                        hint: Some("inspect flash runlog for details".to_string()),
                    });
                    failing_check_id = Some(step_check_id.clone());
                    append_tool_diagnostics(&mut log_lines, &step.command);
                    break;
                }
            }
            Err(err) => {
                log_lines.push(format!("spawn_error: {err}"));
                report.checks.push(Check {
                    status: Status::Fail,
                    check_id: step_check_id.clone(),
                    message: format!("failed to spawn {}: {err}", step.command),
                    hint: Some("verify required tool binaries are installed".to_string()),
                });
                failing_check_id = Some(step_check_id.clone());
                append_tool_diagnostics(&mut log_lines, &step.command);
                break;
            }
        }
    }
    match failing_check_id {
        Some(check_id) => report.checks.push(Check {
            status: Status::Fail,
            check_id: "usb.flash.execute.success".to_string(),
            message: format!("execute failed at step {check_id}"),
            hint: Some("inspect flash runlog for details".to_string()),
        }),
        None => report.checks.push(Check {
            status: Status::Pass,
            check_id: "usb.flash.execute.success".to_string(),
            message: "all execute steps completed successfully".to_string(),
            hint: None,
        }),
    }

    for path in &plan.cleanup_paths {
        let cleanup_target = repo_root.join(path);
        let note = match fs::remove_file(&cleanup_target) {
            Ok(()) => format!("cleanup: removed {}", cleanup_target.display()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                format!("cleanup: not_found {}", cleanup_target.display())
            }
            Err(err) => format!("cleanup: failed {} ({err})", cleanup_target.display()),
        };
        log_lines.push(note);
    }

    if let Err(err) = write_log(log_path, &log_lines.join("\n")) {
        report.checks.push(Check {
            status: Status::Fail,
            check_id: "usb.flash.runlog.write".to_string(),
            message: format!("failed to write runlog: {err}"),
            hint: Some("check --log path permissions".to_string()),
        });
    } else {
        report.checks.push(Check {
            status: Status::Pass,
            check_id: "usb.flash.runlog.write".to_string(),
            message: format!("wrote runlog: {}", log_path.display()),
            hint: None,
        });
        report.checks.push(Check {
            status: Status::Pass,
            check_id: "usb.flash.execute.instrumentation".to_string(),
            message: "execute path instrumentation completed".to_string(),
            hint: None,
        });
    }

    finalize_report(report)
}

fn write_log(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, content).map_err(|err| err.to_string())
}

fn tool_present(command: &str) -> bool {
    match Command::new("which").arg(command).output() {
        Ok(output) => output.status.success(),
        Err(_) => Command::new("sh")
            .args(["-c", "command -v \"$1\"", "sh", command])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false),
    }
}

fn append_tool_diagnostics(log_lines: &mut Vec<String>, command: &str) {
    log_lines.push("tool_diagnostics:".to_string());
    #[cfg(unix)]
    let presence_probe = CommandSpec {
        command: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            "command -v \"$1\"".to_string(),
            "sh".to_string(),
            command.to_string(),
        ],
    };
    #[cfg(windows)]
    let presence_probe = CommandSpec {
        command: "where".to_string(),
        args: vec![command.to_string()],
    };
    append_probe_result(log_lines, &presence_probe);
    let version_probes: [&[&str]; 3] = [&["--version"], &["-V"], &["version"]];
    for args in version_probes {
        let probe = CommandSpec {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        };
        append_probe_result(log_lines, &probe);
    }
}

fn append_probe_result(log_lines: &mut Vec<String>, probe: &CommandSpec) {
    log_lines.push(format!("command: {}", format_command_line(probe)));
    match run_capture(probe) {
        Ok(output) => {
            log_lines.push(format!("status: {}", output.status_code));
            log_lines.push(format!("stdout_bytes: {}", output.stdout_bytes));
            log_lines.push(format!("stderr_bytes: {}", output.stderr_bytes));
            log_lines.push("stdout:".to_string());
            log_lines.extend(output.stdout.lines().map(ToString::to_string));
            log_lines.push("stderr:".to_string());
            log_lines.extend(output.stderr.lines().map(ToString::to_string));
        }
        Err(err) => {
            log_lines.push(format!("spawn_error: {err}"));
        }
    }
}

struct CapturedOutput {
    status_code: i32,
    stdout_bytes: usize,
    stderr_bytes: usize,
    stdout: String,
    stderr: String,
}

fn run_capture(command: &CommandSpec) -> Result<CapturedOutput, String> {
    let output = Command::new(&command.command)
        .args(&command.args)
        .output()
        .map_err(|err| err.to_string())?;
    Ok(CapturedOutput {
        status_code: output.status.code().unwrap_or(1),
        stdout_bytes: output.stdout.len(),
        stderr_bytes: output.stderr.len(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

pub(crate) fn run_interactive(command: &CommandSpec) -> Result<(), String> {
    let status = Command::new(&command.command)
        .args(&command.args)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with {}", command.command, status))
    }
}

pub(crate) fn resolve_board_arg(repo_root: &Path, board_arg: &str) -> Result<PathBuf, Report> {
    if board_arg.contains('/') || board_arg.contains('\\') || board_arg.ends_with(".toml") {
        let as_path = Path::new(board_arg);
        let candidate = if as_path.is_absolute() {
            as_path.to_path_buf()
        } else {
            repo_root.join(as_path)
        };
        let repo_canon = match dunce::canonicalize(repo_root) {
            Ok(canon) => canon,
            Err(err) => {
                return Err(single_check_report(
                    Status::Fail,
                    "board.path_resolve",
                    &format!(
                        "failed to canonicalize repo root '{}': {err}",
                        repo_root.display()
                    ),
                    Some("repository root must be a valid, accessible directory"),
                ))
            }
        };
        match dunce::canonicalize(&candidate) {
            Ok(canon) => {
                if canon.starts_with(&repo_canon) {
                    Ok(canon)
                } else {
                    Err(single_check_report(
                        Status::Fail,
                        "board.path_escape",
                        &format!("board path escapes repo root: {}", candidate.display()),
                        Some("use a board path under the repository root"),
                    ))
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(single_check_report(
                Status::Fail,
                "board.file_missing",
                &format!("board file not found '{}'", candidate.display()),
                Some(BOARD_FILE_MISSING_HINT),
            )),
            Err(err) => Err(single_check_report(
                Status::Fail,
                "board.path_resolve",
                &format!(
                    "failed to resolve board path '{}': {err}",
                    candidate.display()
                ),
                Some("check board path permissions and validity"),
            )),
        }
    } else {
        Ok(repo_root.join("boards").join(format!("{board_arg}.toml")))
    }
}

pub(crate) fn needs_usb_tempdir(plan: &xtask::usb::FlashPlan) -> bool {
    if plan.steps.iter().any(|step| step.command == "rust-objcopy") {
        return true;
    }
    let target_dir = Path::new("target").join("xtask").join("usb");
    if plan
        .cleanup_paths
        .iter()
        .any(|path| path.starts_with(&target_dir))
    {
        return true;
    }
    let marker = "target/xtask/usb/";
    plan.steps
        .iter()
        .flat_map(|step| step.args.iter())
        .any(|arg| arg.contains(marker) || arg.contains("target\\xtask\\usb\\"))
}

fn sanitize_check_id_component(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn board_artifact_name(board_arg: &str, board_path: &Path) -> String {
    if board_arg.contains('/') || board_arg.contains('\\') || board_arg.ends_with(".toml") {
        board_path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("board")
            .to_string()
    } else {
        board_arg.to_string()
    }
}

pub(crate) fn load_board(path: &Path) -> Result<BoardDescriptor, Report> {
    match fs::read_to_string(path) {
        Ok(raw) => match parse_board_descriptor(&raw) {
            Ok(board) => Ok(board),
            Err(err) => Err(single_check_report(
                Status::Fail,
                "board.parse",
                &format!("invalid board descriptor '{}': {err}", path.display()),
                Some("validate board TOML against schema"),
            )),
        },
        Err(err) => Err(single_check_report(
            Status::Fail,
            "board.file_missing",
            &format!("board file not found '{}': {err}", path.display()),
            Some(BOARD_FILE_MISSING_HINT),
        )),
    }
}

pub(crate) fn default_flash_log_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join("docs")
        .join("audits")
        .join("runlogs")
        .join(format!("{}_usb_flash.txt", yyyymmdd_utc()))
}
