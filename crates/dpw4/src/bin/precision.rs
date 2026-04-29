#![forbid(unsafe_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "precision/mod.rs"]
mod precision_app;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    match args.get(1).and_then(|arg| arg.to_str()) {
        None => {
            eprint!("{}", precision_app::minimal_usage_summary());
            std::process::exit(2);
        }
        Some("-h") | Some("--help") => {
            print!("{}", precision_app::help_summary());
            std::process::exit(0);
        }
        Some("-V") | Some("--version") => {
            print!("{}", precision_app::version_summary());
            std::process::exit(0);
        }
        Some(command) if precision_app::is_authoritative_command(command) => {
            common::exit_with_result(precision_app::run(args));
        }
        Some(command) if !command.starts_with('-') => {
            eprint!(
                "unknown command\n{}",
                precision_app::minimal_usage_summary()
            );
            std::process::exit(2);
        }
        Some(_) => {
            common::exit_with_result(precision_app::run(args));
        }
    }
}
