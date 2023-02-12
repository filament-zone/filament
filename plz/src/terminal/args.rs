use std::ffi::OsString;

pub trait Args: Sized {
    fn from_args(args: Vec<OsString>) -> eyre::Result<(Self, Vec<OsString>)>;
}
