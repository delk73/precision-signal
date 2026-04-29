use clap::{error::ErrorKind, Parser};
use getrandom::fill as getrandom_fill;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliStatus {
    Success,
    #[allow(dead_code)]
    Fail,
    Info,
    UserError,
    SystemError,
}

impl CliStatus {
    fn code(self) -> i32 {
        match self {
            CliStatus::Success => 0,
            CliStatus::Fail => 1,
            CliStatus::Info => 0,
            CliStatus::UserError => 2,
            CliStatus::SystemError => 2,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    User(String),
    Integrity(String),
    Clap(clap::Error),
}

pub type CliResult = Result<CliStatus, CliError>;

#[allow(dead_code)]
pub struct AuditState {
    pub bin: &'static str,
    pub version: &'static str,
    pub commit: &'static str,
    pub build_time: &'static str,
    pub toolchain: &'static str,
    pub features: &'static [&'static str],
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> Self {
        CliError::Clap(err)
    }
}

impl CliError {
    fn status(&self) -> CliStatus {
        match self {
            CliError::Io(err) => match err.kind() {
                io::ErrorKind::NotFound
                | io::ErrorKind::InvalidInput
                | io::ErrorKind::InvalidData => CliStatus::UserError,
                io::ErrorKind::PermissionDenied
                | io::ErrorKind::BrokenPipe
                | io::ErrorKind::TimedOut
                | io::ErrorKind::Interrupted
                | io::ErrorKind::AddrInUse
                | io::ErrorKind::AlreadyExists => CliStatus::SystemError,
                _ => CliStatus::SystemError,
            },
            CliError::User(_) | CliError::Integrity(_) => CliStatus::UserError,
            CliError::Clap(err) => match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => CliStatus::Info,
                _ => CliStatus::UserError,
            },
        }
    }
}

#[allow(dead_code)]
pub enum OutHandle {
    File(File),
    Stdout(io::Stdout),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ResultBlock {
    pub result: String,
    pub command: String,
    pub target: String,
    pub mode: String,
    pub equivalence: String,
    pub first_divergence: String,
    pub artifact: String,
}

#[allow(dead_code)]
pub struct ArtifactStaging {
    artifacts_root: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PublishedArtifact {
    pub run_id: String,
    pub relative_artifact_path: String,
    pub final_dir: PathBuf,
    pub result_block: ResultBlock,
}

#[allow(dead_code)]
pub fn audit_state(bin: &'static str) -> AuditState {
    AuditState {
        bin,
        version: env!("CARGO_PKG_VERSION"),
        commit: generated::GIT_HASH,
        build_time: generated::BUILD_TIME,
        toolchain: generated::RUST_VERSION,
        features: generated::FEATURES,
    }
}

#[allow(dead_code)]
pub fn short_commit(hash: &str) -> &str {
    let len = hash.len().min(12);
    &hash[..len]
}

#[allow(dead_code)]
pub fn audit_state_json(bin: &'static str) -> String {
    let state = audit_state(bin);
    let features = state
        .features
        .iter()
        .map(|feature| format!("{feature:?}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"bin\":{:?},\"version\":{:?},\"commit\":{:?},\"build_time\":{:?},\"toolchain\":{:?},\"features\":[{}]}}",
        state.bin, state.version, state.commit, state.build_time, state.toolchain, features
    )
}

#[allow(dead_code)]
impl ResultBlock {
    fn canonicalized(&self, artifact: String) -> Self {
        let is_pass =
            self.equivalence == "exact" && self.first_divergence == "none" && self.result == "PASS";

        Self {
            result: if is_pass {
                "PASS".to_string()
            } else {
                "FAIL".to_string()
            },
            command: self.command.clone(),
            target: self.target.clone(),
            mode: self.mode.clone(),
            equivalence: self.equivalence.clone(),
            first_divergence: self.first_divergence.clone(),
            artifact,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let lines = [
            format!("RESULT: {}", self.result),
            format!("COMMAND: {}", self.command),
            format!("TARGET: {}", self.target),
            format!("MODE: {}", self.mode),
            format!("EQUIVALENCE: {}", self.equivalence),
            format!("FIRST_DIVERGENCE: {}", self.first_divergence),
            format!("ARTIFACT: {}", self.artifact),
        ];

        let mut out = Vec::new();
        for line in lines {
            out.extend_from_slice(line.as_bytes());
            out.push(b'\n');
        }
        out
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), CliError> {
        writer.write_all(&self.to_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn write_to_stdout(&self) -> Result<(), CliError> {
        let mut stdout = io::stdout().lock();
        self.write_to(&mut stdout)
    }
}

#[allow(dead_code)]
impl ArtifactStaging {
    pub fn new<P: Into<PathBuf>>(artifacts_root: P) -> Self {
        Self {
            artifacts_root: artifacts_root.into(),
        }
    }

    pub fn stage_and_publish(
        &self,
        result_block: &ResultBlock,
        trace_json: &[u8],
        meta_json: &[u8],
    ) -> Result<PublishedArtifact, CliError> {
        let run_id = generate_run_id()?;
        self.stage_and_publish_with_run_id(&run_id, result_block, trace_json, meta_json)
    }

    pub fn stage_and_publish_with_run_id(
        &self,
        run_id: &str,
        result_block: &ResultBlock,
        trace_json: &[u8],
        meta_json: &[u8],
    ) -> Result<PublishedArtifact, CliError> {
        fs::create_dir_all(&self.artifacts_root)?;

        let staging_dir = self.artifacts_root.join(format!(".tmp_{run_id}"));
        let final_dir = self.artifacts_root.join(run_id);
        let relative_artifact_path = format!("artifacts/{run_id}");
        let canonical_block = result_block.canonicalized(relative_artifact_path.clone());

        match fs::create_dir(&staging_dir) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                return Err(CliError::Io(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("staging collision for run_id {run_id}"),
                )));
            }
            Err(err) => return Err(err.into()),
        }

        if let Err(err) = self.write_and_publish(
            &staging_dir,
            &final_dir,
            &canonical_block,
            trace_json,
            meta_json,
        ) {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(err);
        }

        Ok(PublishedArtifact {
            run_id: run_id.to_string(),
            relative_artifact_path,
            final_dir,
            result_block: canonical_block,
        })
    }

    pub fn stage_publish_and_emit(
        &self,
        result_block: &ResultBlock,
        trace_json: &[u8],
        meta_json: &[u8],
    ) -> CliResult {
        let published = match self.stage_and_publish(result_block, trace_json, meta_json) {
            Ok(published) => published,
            Err(_) => return Ok(CliStatus::SystemError),
        };
        self.emit_published_result(published)
    }

    fn stage_publish_and_emit_with_run_id(
        &self,
        run_id: &str,
        result_block: &ResultBlock,
        trace_json: &[u8],
        meta_json: &[u8],
    ) -> CliResult {
        let published =
            match self.stage_and_publish_with_run_id(run_id, result_block, trace_json, meta_json) {
                Ok(published) => published,
                Err(_) => return Ok(CliStatus::SystemError),
            };
        self.emit_published_result(published)
    }

    fn emit_published_result(&self, published: PublishedArtifact) -> CliResult {
        let emitted = published.result_block;
        emitted.write_to_stdout()?;

        Ok(match emitted.result.as_str() {
            "PASS" => CliStatus::Success,
            "FAIL" => CliStatus::Fail,
            _ => CliStatus::SystemError,
        })
    }

    fn write_and_publish(
        &self,
        staging_dir: &Path,
        final_dir: &Path,
        result_block: &ResultBlock,
        trace_json: &[u8],
        meta_json: &[u8],
    ) -> Result<(), CliError> {
        write_synced_file(&staging_dir.join("result.txt"), &result_block.to_bytes())?;
        write_synced_file(&staging_dir.join("trace.json"), trace_json)?;
        write_synced_file(&staging_dir.join("meta.json"), meta_json)?;
        fsync_directory(staging_dir)?;
        fs::rename(staging_dir, final_dir)?;
        Ok(())
    }
}

impl Write for OutHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            OutHandle::File(file) => file.write(buf),
            OutHandle::Stdout(stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            OutHandle::File(file) => file.flush(),
            OutHandle::Stdout(stdout) => stdout.flush(),
        }
    }
}

#[allow(dead_code)]
pub(crate) fn parse_args<T>() -> Result<T, CliError>
where
    T: Parser,
{
    match T::try_parse() {
        Ok(value) => Ok(value),
        Err(err) => Err(CliError::Clap(err)),
    }
}

#[allow(dead_code)]
pub(crate) fn parse_from_args<T, I, U>(itr: I) -> Result<T, CliError>
where
    T: Parser,
    I: IntoIterator<Item = U>,
    U: Into<OsString> + Clone,
{
    match T::try_parse_from(itr) {
        Ok(value) => Ok(value),
        Err(err) => Err(CliError::Clap(err)),
    }
}

#[allow(dead_code)]
pub(crate) fn open_output(path: &Path) -> Result<OutHandle, CliError> {
    if path == Path::new("-") {
        Ok(OutHandle::Stdout(io::stdout()))
    } else {
        Ok(OutHandle::File(File::create(path)?))
    }
}

pub(crate) fn exit_with_result(result: CliResult) -> ! {
    match result {
        Ok(status) => std::process::exit(status.code()),
        Err(err) => {
            match &err {
                CliError::Io(io_err) => eprintln!("ERROR: {}", io_err),
                CliError::User(msg) | CliError::Integrity(msg) => eprintln!("ERROR: {}", msg),
                CliError::Clap(clap_err) => match clap_err.kind() {
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => eprint!("{}", clap_err),
                    _ => eprint!("ERROR: {}", clap_err),
                },
            }
            std::process::exit(err.status().code());
        }
    }
}

#[allow(dead_code)]
fn generate_run_id() -> Result<String, CliError> {
    let timestamp = format_safe_id_timestamp(SystemTime::now())?;
    let mut suffix = [0u8; 8];
    getrandom_fill(&mut suffix).map_err(|err| CliError::Io(io::Error::other(err.to_string())))?;
    Ok(format!("{timestamp}-{}", hex::encode(suffix)))
}

#[allow(dead_code)]
fn format_safe_id_timestamp(now: SystemTime) -> Result<String, CliError> {
    let seconds = now
        .duration_since(UNIX_EPOCH)
        .map_err(|err| CliError::Io(io::Error::other(err.to_string())))?
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    Ok(format!(
        "{year:04}{month:02}{day:02}T{hour:02}{minute:02}{second:02}Z"
    ))
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn write_synced_file(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

#[allow(dead_code)]
fn fsync_directory(path: &Path) -> Result<(), CliError> {
    let dir = OpenOptions::new().read(true).open(path)?;
    dir.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("dpw4-common-{label}-{nanos}"))
    }

    fn sample_result_block() -> ResultBlock {
        ResultBlock {
            result: "PASS".to_string(),
            command: "record".to_string(),
            target: "input.bin".to_string(),
            mode: "none".to_string(),
            equivalence: "exact".to_string(),
            first_divergence: "none".to_string(),
            artifact: "artifacts/PLACEHOLDER".to_string(),
        }
    }

    #[test]
    fn result_block_bytes_are_lf_terminated_in_fixed_order() {
        let bytes = sample_result_block().to_bytes();
        let expected = concat!(
            "RESULT: PASS\n",
            "COMMAND: record\n",
            "TARGET: input.bin\n",
            "MODE: none\n",
            "EQUIVALENCE: exact\n",
            "FIRST_DIVERGENCE: none\n",
            "ARTIFACT: artifacts/PLACEHOLDER\n",
        );
        assert_eq!(bytes, expected.as_bytes());
        assert_eq!(bytes.iter().filter(|&&b| b == b'\n').count(), 7);
    }

    #[test]
    fn stage_and_publish_writes_byte_identical_result_file() {
        let root = unique_test_dir("publish");
        let staging = ArtifactStaging::new(&root);
        let block = sample_result_block();
        let published = staging
            .stage_and_publish_with_run_id(
                "20260407T120000Z-0123456789abcdef",
                &block,
                b"{}",
                b"{}",
            )
            .expect("publish should succeed");

        let result_bytes = fs::read(published.final_dir.join("result.txt")).expect("read result");
        assert_eq!(result_bytes, published.result_block.to_bytes());
        assert_eq!(
            published.result_block.artifact,
            "artifacts/20260407T120000Z-0123456789abcdef"
        );
        assert!(published.final_dir.join("trace.json").exists());
        assert!(published.final_dir.join("meta.json").exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn collision_aborts_immediately() {
        let root = unique_test_dir("collision");
        fs::create_dir_all(root.join(".tmp_20260407T120000Z-aaaaaaaaaaaaaaaa"))
            .expect("precreate collision");
        let staging = ArtifactStaging::new(&root);
        let block = sample_result_block();

        let err = staging
            .stage_and_publish_with_run_id(
                "20260407T120000Z-aaaaaaaaaaaaaaaa",
                &block,
                b"{}",
                b"{}",
            )
            .expect_err("collision must fail");
        assert!(
            matches!(err, CliError::Io(io_err) if io_err.kind() == io::ErrorKind::AlreadyExists)
        );
        assert!(!root.join("20260407T120000Z-aaaaaaaaaaaaaaaa").exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rename_failure_suppresses_stdout_and_returns_exit_2() {
        let root = unique_test_dir("rename-fail");
        fs::create_dir_all(&root).expect("root dir");
        let blocking_dir = root.join("20260407T120000Z-deadbeefdeadbeef");
        fs::create_dir_all(&blocking_dir).expect("block final path with directory");
        fs::write(blocking_dir.join("occupied"), b"x").expect("make final dir non-empty");
        let staging = ArtifactStaging::new(&root);
        let block = ResultBlock {
            artifact: "artifacts/20260407T120000Z-deadbeefdeadbeef".to_string(),
            ..sample_result_block()
        };

        let status = staging
            .stage_publish_and_emit_with_run_id(
                "20260407T120000Z-deadbeefdeadbeef",
                &block,
                b"{}",
                b"{}",
            )
            .expect("status");
        assert_eq!(status, CliStatus::SystemError);
        assert!(!root.join(".tmp_20260407T120000Z-deadbeefdeadbeef").exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn canonicalization_forces_fail_and_overrides_artifact() {
        let root = unique_test_dir("canonicalize");
        let staging = ArtifactStaging::new(&root);
        let block = ResultBlock {
            result: "PASS".to_string(),
            equivalence: "diverged".to_string(),
            first_divergence: "step=9 node=osc cause=VAL_MISMATCH".to_string(),
            artifact: "artifacts/UNTRUSTED".to_string(),
            ..sample_result_block()
        };

        let published = staging
            .stage_and_publish_with_run_id(
                "20260407T120000Z-feedfacefeedface",
                &block,
                b"{}",
                b"{}",
            )
            .expect("publish should succeed");
        assert_eq!(published.result_block.result, "FAIL");
        assert_eq!(
            published.result_block.artifact,
            "artifacts/20260407T120000Z-feedfacefeedface"
        );

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn user_and_integrity_errors_map_to_exit_2() {
        let user = CliError::User("bad artifact".to_string());
        let integrity = CliError::Integrity("serialization failed".to_string());

        assert_eq!(user.status(), CliStatus::UserError);
        assert_eq!(integrity.status(), CliStatus::UserError);
        assert_eq!(user.status().code(), 2);
        assert_eq!(integrity.status().code(), 2);
    }
}
