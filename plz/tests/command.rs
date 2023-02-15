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
        .env("PLZ_HOME", home.to_string_lossy())
        .envs(envs)
        .cwd(cwd)
        .file(base.join(test))?
        .run()
        .map(|_| ())?)
}

#[test]
fn plz_usage() -> eyre::Result<()> {
    test("docs/plz-usage.md", Path::new("."), vec![])
}

#[test]
fn plz_version() -> eyre::Result<()> {
    test("docs/plz-version.md", Path::new("."), vec![])
}

#[test]
fn plz_cmd_not_found() -> eyre::Result<()> {
    test("docs/plz-cmd-not-found.md", Path::new("."), vec![])
}
