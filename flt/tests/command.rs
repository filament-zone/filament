use std::{path::Path, vec};

use radicle_cli_test::TestFormula;

fn test<'a>(
    test: impl AsRef<Path>,
    cwd: impl AsRef<Path>,
    envs: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> eyre::Result<()> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().to_path_buf();

    Ok(TestFormula::new()
        .env("FLT_HOME", home.to_string_lossy())
        .envs(envs)
        .cwd(cwd)
        .file(base.join(test))?
        .run()
        .map(|_| ())?)
}

#[test]
fn flt_usage() -> eyre::Result<()> {
    test("docs/flt-usage.md", Path::new("."), vec![])
}

#[test]
fn flt_version() -> eyre::Result<()> {
    test("docs/flt-version.md", Path::new("."), vec![])
}

#[test]
fn flt_cmd_not_found() -> eyre::Result<()> {
    test("docs/flt-cmd-not-found.md", Path::new("."), vec![])
}
