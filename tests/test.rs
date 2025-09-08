use std::process::Command;

const BIN_PATH: &str = env!("CARGO_BIN_EXE_cargo-unused-workspace-deps");

#[test]
fn test_passing_workspace() {
    let output = Command::new(BIN_PATH)
        .arg("unused-workspace-deps")
        .arg("tests/passing-workspace/Cargo.toml")
        .output()
        .expect("Failed to run program");
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn test_failing_workspace() {
    let output = Command::new(BIN_PATH)
        .arg("unused-workspace-deps")
        .arg("tests/failing-workspace/Cargo.toml")
        .output()
        .expect("Failed to run program");
    assert!(!output.status.success(), "{output:?}");
    // Check that it mentions the extra dependency
    assert!(String::from_utf8_lossy(&output.stdout).contains("hashbrown"));
}

#[test]
fn test_workspace_fix() {
    // Set up the workspace for testing
    let tempdir = mktemp::Temp::new_dir().expect("Failed to make temporary directory to run fix");
    assert!(
        Command::new("cp")
            .args([
                "-r",
                "./tests/failing-workspace",
                "./tests/passing-workspace",
                &tempdir.as_ref().to_str().expect("Path wasn't valid utf-8")
            ])
            .status()
            .expect("Failed to copy to workspace")
            .success(),
        "Failed to copy to workspace"
    );
    // Run the fix command on the failing workspace.
    let output = Command::new(BIN_PATH)
        .arg("unused-workspace-deps")
        .arg("--fix")
        .arg(tempdir.as_ref().join("failing-workspace/Cargo.toml"))
        .output()
        .expect("Failed to run program");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(tempdir.as_ref().join("failing-workspace/Cargo.toml")).unwrap(),
        std::fs::read_to_string("./tests/passing-workspace/Cargo.toml").unwrap(),
    );
    // Now check that it passes without the fix command.
    let output = Command::new(BIN_PATH)
        .arg("unused-workspace-deps")
        .arg(tempdir.as_ref().join("failing-workspace/Cargo.toml"))
        .output()
        .expect("Failed to run program");
    assert!(output.status.success(), "{output:?}");
    // Check that the passing workspace isn't changed by the fix command.
    let output = Command::new(BIN_PATH)
        .arg("unused-workspace-deps")
        .arg("--fix")
        .arg(tempdir.as_ref().join("passing-workspace/Cargo.toml"))
        .output()
        .expect("Failed to run program");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(tempdir.as_ref().join("passing-workspace/Cargo.toml")).unwrap(),
        std::fs::read_to_string("./tests/passing-workspace/Cargo.toml").unwrap(),
    );
}
