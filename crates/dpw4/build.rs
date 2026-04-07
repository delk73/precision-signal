use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let dest = out_dir.join("build_info.rs");

    let git_hash = command_output("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let build_time =
        command_output("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]).unwrap_or_else(|| "unknown".to_string());
    let rust_version = command_output("rustc", &["--version"]).unwrap_or_else(|| "unknown".to_string());
    let features = collect_features();

    let generated = format!(
        "pub const GIT_HASH: &str = {git_hash:?};\n\
         pub const BUILD_TIME: &str = {build_time:?};\n\
         pub const RUST_VERSION: &str = {rust_version:?};\n\
         pub const FEATURES: &[&str] = &{features};\n"
    );

    fs::write(dest, generated).expect("write build info");
}

fn command_output(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    Some(value.trim().to_string())
}

fn collect_features() -> String {
    let mut features: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            key.strip_prefix("CARGO_FEATURE_")
                .map(|name| name.to_ascii_lowercase().replace('_', "-"))
        })
        .filter(|feature| feature != "default")
        .collect();
    features.sort();
    features.dedup();

    let quoted: Vec<String> = features.into_iter().map(|feature| format!("{feature:?}")).collect();
    format!("[{}]", quoted.join(", "))
}
