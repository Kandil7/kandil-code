use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use assert_cmd::cargo::cargo_bin;

#[test]
fn help_shows_commands() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Intelligent development platform"));
}

#[test]
fn switch_model_rejects_invalid_provider() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.args(["switch-model", "invalid", "gpt-4"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid provider"));
}

#[test]
fn config_costs_shows_message() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.args(["config", "costs"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Cost tracking is available when using AI features"));
}

#[test]
fn config_validate_ok_with_defaults() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.args(["config", "validate"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration validation: ok"));
}

#[test]
fn config_validate_fails_with_unknown_provider() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.env("KANDIL_AI_PROVIDER", "unknown");
    cmd.args(["config", "validate"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported AI provider"));
}

#[test]
fn local_model_status_runs() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.args(["local-model", "status"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Ollama"));
}

#[test]
fn local_model_use_persists() {
    let mut cmd = Command::from(cargo_bin!("kandil"));
    cmd.args(["local-model", "use", "llama3:8b"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Using local model"));
}
