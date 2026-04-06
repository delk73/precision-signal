use clap::{error::ErrorKind, Parser};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliStatus {
    Success,
    Info,
    UserError,
    SystemError,
}

impl CliStatus {
    fn code(self) -> i32 {
        match self {
            CliStatus::Success => 0,
            CliStatus::Info => 0,
            CliStatus::UserError => 2,
            CliStatus::SystemError => 1,
        }
    }
}

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    User(String),
    Integrity(String),
    Clap(clap::Error),
}

pub type CliResult = Result<CliStatus, CliError>;

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
                | io::ErrorKind::PermissionDenied
                | io::ErrorKind::InvalidInput
                | io::ErrorKind::InvalidData => CliStatus::UserError,
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

pub enum OutHandle {
    File(File),
    Stdout(io::Stdout),
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
                CliError::Clap(clap_err) => eprint!("{}", clap_err),
            }
            std::process::exit(err.status().code());
        }
    }
}
