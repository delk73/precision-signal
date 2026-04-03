use crate::docs::emit_report;
use crate::firmware::{
    board_artifact_name, default_flash_log_path, load_board, resolve_board_arg, run_flash_execute,
    run_interactive, stlink_image_compat_plan,
};
use crate::replay::usage;
use crate::util::{discover_repo_root, ensure_empty, take_flag, take_optional_value, take_value};
use crate::workflow;
use std::path::{Path, PathBuf};
use xtask::board::PreferredBackend;
use xtask::usb::{
    debug_plan, doctor_report, empty_report, finalize_report, flash_plan, format_command_line,
    plan_report, single_check_report, Check, Event, Report, Status,
};

const EXIT_OK: i32 = 0;
const EXIT_FAIL: i32 = 1;
const EXIT_USAGE: i32 = 2;
pub(crate) const BOARD_FILE_MISSING_HINT: &str =
    "use --board <id> or pass a path under the repository root";

#[derive(Debug)]
pub(crate) struct UsageError(pub(crate) String);

pub(crate) struct RunOutcome {
    report: Report,
    json: bool,
    verbose: bool,
}

pub(crate) fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo_root = discover_repo_root(&cwd);
    let exit_code = run(std::env::args().skip(1).collect(), &repo_root);
    std::process::exit(exit_code);
}

pub(crate) fn run(args: Vec<String>, repo_root: &Path) -> i32 {
    if args.is_empty() || matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        println!("{}", top_level_usage());
        return EXIT_OK;
    }

    if args.first().map(String::as_str) == Some("workflow") {
        return workflow::run(args[1..].to_vec(), repo_root);
    }

    let run = run_internal(args, repo_root);
    match run {
        Ok(outcome) => {
            if let Err(err) = emit_report(&outcome.report, outcome.json, outcome.verbose) {
                eprintln!("{err}");
                return EXIT_USAGE;
            }
            if outcome.report.overall == Status::Fail {
                EXIT_FAIL
            } else {
                EXIT_OK
            }
        }
        Err(UsageError(err)) => {
            eprintln!("{err}");
            EXIT_USAGE
        }
    }
}

pub(crate) fn run_internal(args: Vec<String>, repo_root: &Path) -> Result<RunOutcome, UsageError> {
    if args.first().map(String::as_str) != Some("usb") {
        return Err(UsageError(top_level_usage()));
    }
    if args.len() < 2 {
        return Err(UsageError(usage()));
    }

    let command = args[1].as_str();
    let mut rest = args[2..].to_vec();
    let json = take_flag(&mut rest, "--json");
    let verbose = take_flag(&mut rest, "--verbose");

    match command {
        "doctor" => {
            let board_arg = take_value(&mut rest, "--board")?;
            if take_flag(&mut rest, "--execute") {
                return Err(UsageError(
                    "--execute is not valid for usb doctor".to_string(),
                ));
            }
            ensure_empty(&rest)?;
            let board_path = match resolve_board_arg(repo_root, &board_arg) {
                Ok(path) => path,
                Err(report) => {
                    return Ok(RunOutcome {
                        report,
                        json,
                        verbose,
                    })
                }
            };
            let report = match load_board(&board_path) {
                Ok(board) => doctor_report(&board, verbose),
                Err(report) => report,
            };
            Ok(RunOutcome {
                report,
                json,
                verbose,
            })
        }
        "flash" => {
            let board_arg = take_value(&mut rest, "--board")?;
            let elf = take_optional_value(&mut rest, "--elf")?;
            let image = take_optional_value(&mut rest, "--image")?;
            let log = take_optional_value(&mut rest, "--log")?;
            let execute = take_flag(&mut rest, "--execute");
            ensure_empty(&rest)?;
            if elf.is_none() && image.is_none() {
                return Ok(RunOutcome {
                    report: single_check_report(
                        Status::Fail,
                        "cli.elf_required",
                        "missing required flash input",
                        Some("use --elf <path/to/fw.elf>"),
                    ),
                    json,
                    verbose,
                });
            }
            if elf.is_some() && image.is_some() {
                return Err(UsageError(
                    "provide only one of --elf or --image".to_string(),
                ));
            }

            let board_path = match resolve_board_arg(repo_root, &board_arg) {
                Ok(path) => path,
                Err(report) => {
                    return Ok(RunOutcome {
                        report,
                        json,
                        verbose,
                    })
                }
            };
            let board = match load_board(&board_path) {
                Ok(board) => board,
                Err(report) => {
                    return Ok(RunOutcome {
                        report,
                        json,
                        verbose,
                    })
                }
            };
            let board_name = board_artifact_name(&board_arg, &board_path);

            let mut report = empty_report();
            let input_path: String;
            if let Some(image_path) = image.as_ref() {
                if board.preferred_backend != PreferredBackend::Stlink {
                    report.checks.push(Check {
                        status: Status::Fail,
                        check_id: "cli.image_unsupported_backend".to_string(),
                        message: "--image is only supported for stlink and .bin inputs".to_string(),
                        hint: Some("use --elf <path> instead".to_string()),
                    });
                    return Ok(RunOutcome {
                        report: finalize_report(report),
                        json,
                        verbose,
                    });
                }
                if !image_path.ends_with(".bin") {
                    report.checks.push(Check {
                        status: Status::Fail,
                        check_id: "cli.image_requires_bin".to_string(),
                        message: "--image requires a .bin file for stlink".to_string(),
                        hint: Some("use --elf <path> instead".to_string()),
                    });
                    return Ok(RunOutcome {
                        report: finalize_report(report),
                        json,
                        verbose,
                    });
                }
                report.checks.push(Check {
                    status: Status::Warn,
                    check_id: "cli.deprecated_image_flag".to_string(),
                    message: "--image is deprecated".to_string(),
                    hint: Some("use --elf <path> instead".to_string()),
                });
                input_path = image_path.clone();
            } else {
                input_path = elf.expect("validated above");
                if !Path::new(&input_path).exists() {
                    let fail = single_check_report(
                        Status::Fail,
                        "cli.elf_file_missing",
                        &format!("ELF file does not exist: {input_path}"),
                        Some("provide a valid --elf path"),
                    );
                    return Ok(RunOutcome {
                        report: fail,
                        json,
                        verbose,
                    });
                }
            }

            if image.as_ref().is_some() {
                if !Path::new(&input_path).exists() {
                    let fail = single_check_report(
                        Status::Fail,
                        "cli.image_file_missing",
                        &format!("image file does not exist: {input_path}"),
                        Some("provide a valid --image path"),
                    );
                    return Ok(RunOutcome {
                        report: fail,
                        json,
                        verbose,
                    });
                }
            }

            let mut plan = flash_plan(&board, &input_path, &board_name);
            if image.as_ref().is_some() {
                plan = stlink_image_compat_plan(&board, &input_path);
            }
            let mut plan_report_out = plan_report("flash", &plan.steps, verbose);
            plan_report_out.checks.append(&mut report.checks);
            let mut combined = finalize_report(plan_report_out);

            if execute {
                let log_path = log
                    .map(PathBuf::from)
                    .unwrap_or_else(|| default_flash_log_path(repo_root));
                let exec_report = run_flash_execute(repo_root, &plan, &log_path, verbose);
                combined.checks.extend(exec_report.checks);
                combined.events.extend(exec_report.events);
                combined = finalize_report(combined);
            }

            Ok(RunOutcome {
                report: combined,
                json,
                verbose,
            })
        }
        "debug" => {
            let board_arg = take_value(&mut rest, "--board")?;
            let execute = take_flag(&mut rest, "--execute");
            ensure_empty(&rest)?;

            let board_path = match resolve_board_arg(repo_root, &board_arg) {
                Ok(path) => path,
                Err(report) => {
                    return Ok(RunOutcome {
                        report,
                        json,
                        verbose,
                    })
                }
            };
            let board = match load_board(&board_path) {
                Ok(board) => board,
                Err(report) => {
                    return Ok(RunOutcome {
                        report,
                        json,
                        verbose,
                    })
                }
            };
            let plan = debug_plan(&board);
            let mut report = plan_report("debug", std::slice::from_ref(&plan.spawn), verbose);
            report.checks.push(Check {
                status: Status::Pass,
                check_id: "usb.debug.attach_hint".to_string(),
                message: plan.attach_hint.clone(),
                hint: None,
            });
            report = finalize_report(report);

            if execute {
                if verbose {
                    report.events.push(Event {
                        kind: "spawn_command_line".to_string(),
                        text: format_command_line(&plan.spawn),
                    });
                }
                match run_interactive(&plan.spawn) {
                    Ok(()) => {
                        report.checks.push(Check {
                            status: Status::Pass,
                            check_id: "usb.debug.execute".to_string(),
                            message: "debug server exited successfully".to_string(),
                            hint: None,
                        });
                    }
                    Err(err) => {
                        report.checks.push(Check {
                            status: Status::Fail,
                            check_id: "usb.debug.execute".to_string(),
                            message: err,
                            hint: Some(
                                "verify backend tools are installed and reachable".to_string(),
                            ),
                        });
                    }
                }
                report = finalize_report(report);
            }

            Ok(RunOutcome {
                report,
                json,
                verbose,
            })
        }
        _ => Err(UsageError(usage())),
    }
}

fn top_level_usage() -> String {
    "usage: cargo xtask usb <command> [options]\n       cargo xtask workflow <command>\n\ncommands:\n  usb       operator USB workflows (doctor, flash, debug)\n  workflow  repository verification and replay workflows"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{run, run_internal, EXIT_FAIL, EXIT_OK, EXIT_USAGE};
    use crate::firmware::{
        needs_usb_tempdir, resolve_board_arg, run_flash_execute, stlink_image_compat_plan,
    };
    use crate::util::discover_repo_root;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use xtask::board::parse_board_descriptor;
    use xtask::usb::{flash_plan, CommandSpec, FlashPlan, Status};

    fn temp_test_dir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "xtask_usb_test_{}_{}_{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create temp test dir");
        base
    }

    #[test]
    fn board_arg_id_and_path_resolution() {
        let root = temp_test_dir("board_path_resolution");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        fs::write(
            root.join("boards/custom.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write custom board");
        fs::write(
            root.join("custom.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write custom root board");
        assert_eq!(
            resolve_board_arg(&root, "nucleo-f446re").expect("id path"),
            root.join("boards/nucleo-f446re.toml")
        );
        assert_eq!(
            resolve_board_arg(&root, "boards/custom.toml").expect("relative path"),
            dunce::canonicalize(root.join("boards/custom.toml")).expect("canonical board path")
        );
        assert_eq!(
            resolve_board_arg(&root, "custom.toml").expect("relative path"),
            dunce::canonicalize(root.join("custom.toml")).expect("canonical root path")
        );
    }

    #[test]
    fn board_path_style_relative_escape_is_rejected() {
        let base = temp_test_dir("path_escape_base");
        let root = base.join("repo");
        fs::create_dir_all(&root).expect("repo dir");
        let outside = base.join("outside.toml");
        fs::write(
            &outside,
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write outside board");
        let report = resolve_board_arg(&root, "../outside.toml").expect_err("should reject escape");
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "board.path_escape"),
            "expected board.path_escape check"
        );
    }

    #[test]
    fn board_path_style_absolute_inside_repo_passes() {
        let root = temp_test_dir("abs_inside");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        let inside = root.join("boards/inside.toml");
        fs::write(
            &inside,
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write board");
        let resolved = resolve_board_arg(&root, inside.to_string_lossy().as_ref())
            .expect("absolute inside path");
        assert_eq!(
            resolved,
            dunce::canonicalize(&inside).expect("canonical inside")
        );
    }

    #[test]
    fn board_path_style_absolute_outside_repo_is_rejected() {
        let base = temp_test_dir("abs_outside_base");
        let root = base.join("repo");
        fs::create_dir_all(&root).expect("repo dir");
        let outside = base.join("outside.toml");
        fs::write(
            &outside,
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write outside board");
        let report = resolve_board_arg(&root, outside.to_string_lossy().as_ref())
            .expect_err("absolute outside should fail");
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "board.path_escape"),
            "expected board.path_escape check"
        );
    }

    #[cfg(unix)]
    #[test]
    fn board_path_style_symlink_outside_repo_is_rejected() {
        let base = temp_test_dir("symlink_outside_base");
        let root = base.join("repo");
        let boards = root.join("boards");
        fs::create_dir_all(&boards).expect("boards dir");
        let outside = base.join("outside.toml");
        fs::write(
            &outside,
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write outside board");
        let link = boards.join("link.toml");
        std::os::unix::fs::symlink(&outside, &link).expect("create symlink");

        let report =
            resolve_board_arg(&root, "boards/link.toml").expect_err("symlink escape should fail");
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "board.path_escape"),
            "expected board.path_escape check"
        );
    }

    #[test]
    fn board_path_style_missing_file_reports_board_file_missing() {
        let root = temp_test_dir("path_missing");
        let rel_report = resolve_board_arg(&root, "boards/missing.toml")
            .expect_err("relative missing should fail");
        assert!(
            rel_report
                .checks
                .iter()
                .any(|c| c.check_id == "board.file_missing"),
            "expected board.file_missing for relative path"
        );

        let abs_missing = root.join("missing-abs.toml");
        let abs_report = resolve_board_arg(&root, abs_missing.to_string_lossy().as_ref())
            .expect_err("absolute missing should fail");
        assert!(
            abs_report
                .checks
                .iter()
                .any(|c| c.check_id == "board.file_missing"),
            "expected board.file_missing for absolute path"
        );
    }

    #[test]
    fn board_path_style_repo_root_canonicalize_failure_reports_path_resolve() {
        let root = temp_test_dir("repo_root_missing");
        let board = root.join("board.toml");
        fs::write(
            &board,
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write board");
        fs::remove_dir_all(&root).expect("remove repo root");
        let report = resolve_board_arg(&root, board.to_string_lossy().as_ref())
            .expect_err("missing repo root should fail");
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "board.path_resolve"),
            "expected board.path_resolve check"
        );
    }

    #[cfg(windows)]
    #[test]
    fn board_path_style_windows_dot_segments_normalize_inside_repo() {
        let root = temp_test_dir("windows_dot_segments");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        let inside = root.join("boards/inside.toml");
        fs::write(
            &inside,
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write board");
        let arg = ".\\boards\\..\\boards\\inside.toml";
        let resolved = resolve_board_arg(&root, arg).expect("dot-segment path should resolve");
        assert_eq!(
            resolved,
            dunce::canonicalize(&inside).expect("canonical inside")
        );
    }

    #[test]
    fn exit_code_policy_fail_and_usage() {
        let root = temp_test_dir("exit_codes");
        let usage_code = run(vec!["usb".to_string()], &root);
        assert_eq!(usage_code, EXIT_USAGE);

        let fail_code = run(
            vec![
                "usb".to_string(),
                "doctor".to_string(),
                "--board".to_string(),
                "missing".to_string(),
            ],
            &root,
        );
        assert_eq!(fail_code, EXIT_FAIL);
    }

    #[test]
    fn top_level_help_paths_exit_zero() {
        let root = temp_test_dir("top_level_help");
        assert_eq!(run(Vec::new(), &root), EXIT_OK);
        assert_eq!(run(vec!["--help".to_string()], &root), EXIT_OK);
        assert_eq!(run(vec!["-h".to_string()], &root), EXIT_OK);
    }

    #[test]
    fn exit_code_policy_warn_or_pass_is_zero() {
        let root = temp_test_dir("warn_zero");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        fs::write(
            root.join("boards/test.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write board");
        let code = run(
            vec![
                "usb".to_string(),
                "doctor".to_string(),
                "--board".to_string(),
                "test".to_string(),
            ],
            &root,
        );
        assert_eq!(code, EXIT_OK);
    }

    #[test]
    fn flash_missing_elf_is_fail_report_exit1() {
        let root = temp_test_dir("flash_missing_elf");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        fs::write(
            root.join("boards/test.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write board");
        let code = run(
            vec![
                "usb".to_string(),
                "flash".to_string(),
                "--board".to_string(),
                "test".to_string(),
            ],
            &root,
        );
        assert_eq!(code, EXIT_FAIL);
    }

    #[test]
    fn repo_root_discovery_works_from_subdir() {
        let root = temp_test_dir("discover_root");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        fs::create_dir_all(root.join("a/b/c")).expect("subdir");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").expect("workspace cargo");
        fs::write(
            root.join("boards/test.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write board");
        let subdir = root.join("a/b/c");
        let discovered = discover_repo_root(&subdir);
        assert_eq!(discovered, root);
        let code = run(
            vec![
                "usb".to_string(),
                "doctor".to_string(),
                "--board".to_string(),
                "test".to_string(),
            ],
            &discovered,
        );
        assert_eq!(code, EXIT_OK);
    }

    #[test]
    fn needs_usb_tempdir_true_for_stlink_elf_false_for_openocd() {
        let stlink = parse_board_descriptor(
            r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "stlink"
"#,
        )
        .expect("stlink board parse");
        let stlink_plan = flash_plan(&stlink, "target/fw.elf", "spot");
        assert!(needs_usb_tempdir(&stlink_plan));

        let openocd = parse_board_descriptor(
            r#"
chip = "STM32F446"
flash_base = 0x08000000
preferred_backend = "openocd"
"#,
        )
        .expect("openocd board parse");
        let openocd_plan = flash_plan(&openocd, "target/fw.elf", "spot");
        assert!(!needs_usb_tempdir(&openocd_plan));

        let compat_plan = stlink_image_compat_plan(&stlink, "fw.bin");
        assert!(!needs_usb_tempdir(&compat_plan));
        assert!(Path::new("fw.bin").is_relative());
    }

    #[test]
    fn needs_usb_tempdir_true_for_objcopy_semantics_without_markers() {
        let root = temp_test_dir("objcopy_semantics");
        let fw = root.join("fw.elf");
        let out = root.join("output").join("fw.bin");
        let plan = FlashPlan {
            steps: vec![CommandSpec {
                command: "rust-objcopy".to_string(),
                args: vec![
                    "-O".to_string(),
                    "binary".to_string(),
                    fw.to_string_lossy().to_string(),
                    out.to_string_lossy().to_string(),
                ],
            }],
            cleanup_paths: Vec::new(),
        };
        assert!(needs_usb_tempdir(&plan));
    }

    #[test]
    fn image_fail_check_ids_are_split() {
        let root = temp_test_dir("image_ids");
        fs::create_dir_all(root.join("boards")).expect("boards dir");
        fs::write(
            root.join("boards/open.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"openocd\"\n",
        )
        .expect("write openocd board");
        fs::write(
            root.join("boards/st.toml"),
            "chip = \"STM32F446\"\nflash_base = 0x08000000\npreferred_backend = \"stlink\"\n",
        )
        .expect("write stlink board");

        let unsupported = run_internal(
            vec![
                "usb".to_string(),
                "flash".to_string(),
                "--board".to_string(),
                "open".to_string(),
                "--image".to_string(),
                "/tmp/fw.bin".to_string(),
            ],
            &root,
        )
        .expect("outcome");
        assert!(unsupported
            .report
            .checks
            .iter()
            .any(|c| c.check_id == "cli.image_unsupported_backend"));

        let requires_bin = run_internal(
            vec![
                "usb".to_string(),
                "flash".to_string(),
                "--board".to_string(),
                "st".to_string(),
                "--image".to_string(),
                "/tmp/fw.elf".to_string(),
            ],
            &root,
        )
        .expect("outcome");
        assert!(requires_bin
            .report
            .checks
            .iter()
            .any(|c| c.check_id == "cli.image_requires_bin"));
    }

    #[test]
    fn flash_execute_step_check_ids_are_unique_per_step_and_backend_shape() {
        let log_path =
            std::env::temp_dir().join(format!("xtask_runlog_{}.txt", std::process::id()));
        let repo_root = temp_test_dir("exec_step_ids");

        let stlink_plan = FlashPlan {
            steps: vec![
                CommandSpec {
                    command: "true".to_string(),
                    args: Vec::new(),
                },
                CommandSpec {
                    command: "true".to_string(),
                    args: Vec::new(),
                },
            ],
            cleanup_paths: Vec::new(),
        };
        let stlink_report = run_flash_execute(&repo_root, &stlink_plan, &log_path, false);
        assert!(stlink_report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute_step.00.true"));
        assert!(stlink_report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute_step.01.true"));

        let openocd_plan = FlashPlan {
            steps: vec![CommandSpec {
                command: "true".to_string(),
                args: Vec::new(),
            }],
            cleanup_paths: Vec::new(),
        };
        let openocd_report = run_flash_execute(&repo_root, &openocd_plan, &log_path, false);
        assert!(openocd_report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute_step.00.true"));
        assert!(!openocd_report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute_step.01.true"));
    }

    #[test]
    fn runlog_contains_stdout_stderr_byte_counts() {
        let log_path =
            std::env::temp_dir().join(format!("xtask_runlog_bytes_{}.txt", std::process::id()));
        let repo_root = temp_test_dir("runlog_bytes");
        let plan = FlashPlan {
            steps: vec![CommandSpec {
                command: "true".to_string(),
                args: Vec::new(),
            }],
            cleanup_paths: Vec::new(),
        };
        let _ = run_flash_execute(&repo_root, &plan, &log_path, false);
        let content = fs::read_to_string(&log_path).expect("read runlog");
        assert!(content.contains("stdout_bytes: 0"));
        assert!(content.contains("stderr_bytes: 0"));
        let _ = fs::remove_file(log_path);
    }

    #[test]
    fn execute_fail_message_includes_exit_code_and_instrumentation_check() {
        let log_path =
            std::env::temp_dir().join(format!("xtask_runlog_fail_{}.txt", std::process::id()));
        let repo_root = temp_test_dir("runlog_fail_exit_code");
        let plan = FlashPlan {
            steps: vec![CommandSpec {
                command: "false".to_string(),
                args: Vec::new(),
            }],
            cleanup_paths: Vec::new(),
        };

        let report = run_flash_execute(&repo_root, &plan, &log_path, false);

        assert!(report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute_step.00.false"
                && c.message == "non-zero exit from false (exit=1)"));
        assert!(report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.runlog.write" && c.status == Status::Pass));
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "usb.flash.execute.instrumentation"
                    && c.status == Status::Pass)
        );
        assert!(report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute.success"
                && c.status == Status::Fail
                && c.message.contains("usb.flash.execute_step.00.false")));
        assert_eq!(report.overall, Status::Fail);
        let content = fs::read_to_string(&log_path).expect("read runlog");
        assert!(content.contains("tool_diagnostics:"));
        assert!(content.contains("command: false --version"));

        let _ = fs::remove_file(log_path);
    }

    #[test]
    fn execute_success_check_is_present_on_all_steps_success() {
        let log_path =
            std::env::temp_dir().join(format!("xtask_runlog_success_{}.txt", std::process::id()));
        let repo_root = temp_test_dir("runlog_success");
        let plan = FlashPlan {
            steps: vec![CommandSpec {
                command: "true".to_string(),
                args: Vec::new(),
            }],
            cleanup_paths: Vec::new(),
        };

        let report = run_flash_execute(&repo_root, &plan, &log_path, false);
        assert!(report
            .checks
            .iter()
            .any(|c| c.check_id == "usb.flash.execute.success"
                && c.status == Status::Pass
                && c.message == "all execute steps completed successfully"));
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.check_id == "usb.flash.execute.instrumentation"
                    && c.status == Status::Pass)
        );
        assert_eq!(report.overall, Status::Pass);

        let _ = fs::remove_file(log_path);
    }

    #[test]
    fn tool_diagnostics_presence_probe_command_is_platform_correct() {
        let log_path =
            std::env::temp_dir().join(format!("xtask_runlog_probe_{}.txt", std::process::id()));
        let repo_root = temp_test_dir("runlog_probe_command");
        let tool = "false";
        let plan = FlashPlan {
            steps: vec![CommandSpec {
                command: tool.to_string(),
                args: Vec::new(),
            }],
            cleanup_paths: Vec::new(),
        };

        let _ = run_flash_execute(&repo_root, &plan, &log_path, false);
        let content = fs::read_to_string(&log_path).expect("read runlog");
        #[cfg(unix)]
        assert!(content.contains(&format!(
            "command: sh -c \"command -v \\\"$1\\\"\" sh {tool}"
        )));
        #[cfg(windows)]
        assert!(content.contains(&format!("command: where {tool}")));

        let _ = fs::remove_file(log_path);
    }

    #[test]
    fn tool_diagnostics_version_probes_are_present_and_ordered() {
        let log_path = std::env::temp_dir().join(format!(
            "xtask_runlog_probe_order_{}.txt",
            std::process::id()
        ));
        let repo_root = temp_test_dir("runlog_probe_order");
        let tool = "false";
        let plan = FlashPlan {
            steps: vec![CommandSpec {
                command: tool.to_string(),
                args: Vec::new(),
            }],
            cleanup_paths: Vec::new(),
        };

        let _ = run_flash_execute(&repo_root, &plan, &log_path, false);
        let content = fs::read_to_string(&log_path).expect("read runlog");
        #[cfg(unix)]
        let presence = format!("command: sh -c \"command -v \\\"$1\\\"\" sh {tool}");
        #[cfg(windows)]
        let presence = format!("command: where {tool}");
        let cmd_vv = format!("command: {tool} --version");
        let cmd_v = format!("command: {tool} -V");
        let cmd_word = format!("command: {tool} version");

        let i_presence = content.find(&presence).expect("presence probe");
        let i_vv = content.find(&cmd_vv).expect("--version probe");
        let i_v = content.find(&cmd_v).expect("-V probe");
        let i_word = content.find(&cmd_word).expect("version probe");
        assert!(i_presence < i_vv);
        assert!(i_vv < i_v);
        assert!(i_v < i_word);

        let _ = fs::remove_file(log_path);
    }
}
