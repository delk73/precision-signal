use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let dest = out_dir.join("build_info.rs");

    let git_hash = command_output("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let build_time = reproducible_build_time().unwrap_or_else(|| "unknown".to_string());
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

fn reproducible_build_time() -> Option<String> {
    if let Ok(epoch) = env::var("SOURCE_DATE_EPOCH") {
        let seconds = epoch.trim().parse::<i64>().ok()?;
        return Some(unix_seconds_to_rfc3339(seconds));
    }

    command_output("git", &["show", "-s", "--format=%cI", "HEAD"])
}

fn unix_seconds_to_rfc3339(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);

    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}
