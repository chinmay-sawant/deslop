use std::process::{Command, Output};

pub(crate) fn cargo_bin() -> String {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_deslop") {
        return path;
    }

    let mut path = std::env::current_exe().expect("test binary path should be available");
    path.pop();
    path.pop();
    path.push("deslop");
    path.to_string_lossy().into_owned()
}

pub(crate) fn run_cli(args: &[&str]) -> Output {
    Command::new(cargo_bin())
        .args(args)
        .output()
        .expect("cli command should execute")
}

pub(crate) fn parse_json_output(output: &Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).expect("JSON output should be valid JSON")
}
