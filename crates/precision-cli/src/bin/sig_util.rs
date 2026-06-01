#![forbid(unsafe_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "sig_util/mod.rs"]
mod sig_util_app;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    match args.get(1).and_then(|arg| arg.to_str()) {
        None => {
            eprint!("{}", sig_util_app::minimal_usage_summary());
            std::process::exit(2);
        }
        Some("-h") | Some("--help") => {
            print!("{}", sig_util_app::help_summary());
            std::process::exit(0);
        }
        Some("-V") | Some("--version") => {
            print!("{}", sig_util_app::version_summary());
            std::process::exit(0);
        }
        Some(_) => {
            common::exit_with_result(sig_util_app::run());
        }
    }
}
