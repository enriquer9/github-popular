use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("github-popular").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("github-popular"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--toml"));
}

#[test]
fn test_cli_missing_account_argument() {
    let mut cmd = Command::cargo_bin("github-popular").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_duplicate_json_and_toml_paths() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("github-popular").unwrap();
    cmd.arg("rust-lang")
        .arg("--json")
        .arg(path)
        .arg("--toml")
        .arg(path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Conflicting export paths"));
}
