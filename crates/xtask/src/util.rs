use crate::release::UsageError;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

pub(crate) fn take_value(args: &mut Vec<String>, key: &str) -> Result<String, UsageError> {
    let value = take_optional_value(args, key)?;
    value.ok_or_else(|| UsageError(format!("missing required argument: {key}")))
}

pub(crate) fn take_optional_value(
    args: &mut Vec<String>,
    key: &str,
) -> Result<Option<String>, UsageError> {
    let Some(index) = args.iter().position(|arg| arg == key) else {
        return Ok(None);
    };
    if index + 1 >= args.len() {
        return Err(UsageError(format!("missing value for argument: {key}")));
    }
    let value = args.remove(index + 1);
    args.remove(index);
    Ok(Some(value))
}

pub(crate) fn ensure_empty(rest: &[String]) -> Result<(), UsageError> {
    if rest.is_empty() {
        Ok(())
    } else {
        Err(UsageError(format!(
            "unexpected arguments: {}",
            rest.join(" ")
        )))
    }
}

pub(crate) fn discover_repo_root(cwd: &Path) -> PathBuf {
    for dir in limited_ancestors(cwd, 8) {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.is_file()
            && std::fs::read_to_string(&cargo_toml)
                .map(|content| content.contains("[workspace]"))
                .unwrap_or(false)
        {
            return dir.to_path_buf();
        }
    }
    for dir in limited_ancestors(cwd, 8) {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
    }
    cwd.to_path_buf()
}

fn limited_ancestors<'a>(start: &'a Path, max_levels: usize) -> Vec<&'a Path> {
    let mut dirs = Vec::new();
    let mut current = Some(start);
    for _ in 0..=max_levels {
        let Some(dir) = current else {
            break;
        };
        dirs.push(dir);
        current = dir.parent();
    }
    dirs
}

pub(crate) fn yyyymmdd_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (seconds / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}{month:02}{day:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    y += if m <= 2 { 1 } else { 0 };
    (y as i32, m as u32, d as u32)
}
