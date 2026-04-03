use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::model::Severity;
use crate::{DEFAULT_MAX_BYTES, read_to_string_limited};

const CONFIG_FILE_NAME: &str = ".deslop.toml";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(crate) struct RepoConfig {
    pub go_semantic_experimental: bool,
    pub rust_async_experimental: bool,
    pub disabled_rules: Vec<String>,
    pub suppressed_paths: Vec<PathBuf>,
    pub severity_overrides: BTreeMap<String, Severity>,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            go_semantic_experimental: false,
            rust_async_experimental: true,
            disabled_rules: Vec::new(),
            suppressed_paths: Vec::new(),
            severity_overrides: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read config {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

pub(crate) fn load_repository_config(root: &Path) -> Result<RepoConfig, Error> {
    let path = root.join(CONFIG_FILE_NAME);
    if !path.is_file() {
        return Ok(RepoConfig::default());
    }

    let text = read_to_string_limited(&path, DEFAULT_MAX_BYTES).map_err(|error| match error {
        crate::Error::Io { path, source } => Error::Read { path, source },
        other => Error::Read {
            path: path.clone(),
            source: std::io::Error::other(other.to_string()),
        },
    })?;
    toml::from_str(&text).map_err(|source| Error::Parse { path, source })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{RepoConfig, load_repository_config};
    use crate::model::Severity;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("deslop-config-{name}-{nonce}"));
        fs::create_dir_all(&path).expect("config temp dir should be created");
        path
    }

    #[test]
    fn returns_default_when_config_file_is_missing() {
        let root = temp_dir("missing");
        let config = load_repository_config(&root).expect("missing config should default");
        assert_eq!(config, RepoConfig::default());
        fs::remove_dir_all(root).expect("config temp dir should be removed");
    }

    #[test]
    fn loads_rule_settings_from_toml() {
        let root = temp_dir("toml");
        fs::write(
            root.join(".deslop.toml"),
            "go_semantic_experimental = true\nrust_async_experimental = false\ndisabled_rules = [\"panic_macro_leftover\"]\nsuppressed_paths = [\"tests/fixtures\"]\n[severity_overrides]\nunwrap_in_non_test_code = \"error\"\n",
        )
        .expect("config file should be written");

        let config = load_repository_config(&root).expect("config should parse");
        assert!(config.go_semantic_experimental);
        assert!(!config.rust_async_experimental);
        assert_eq!(
            config.disabled_rules,
            vec!["panic_macro_leftover".to_string()]
        );
        assert_eq!(
            config.suppressed_paths,
            vec![PathBuf::from("tests/fixtures")]
        );
        assert_eq!(
            config.severity_overrides.get("unwrap_in_non_test_code"),
            Some(&Severity::Error)
        );

        fs::remove_dir_all(root).expect("config temp dir should be removed");
    }

    #[test]
    fn rejects_malformed_toml_with_parse_error() {
        let root = temp_dir("malformed");
        fs::write(
            root.join(".deslop.toml"),
            "this is not valid toml {{{\n",
        )
        .expect("config file should be written");

        let error = load_repository_config(&root).expect_err("malformed TOML should fail");
        assert!(
            matches!(error, super::Error::Parse { .. }),
            "error should be Parse variant, got: {error:?}"
        );

        fs::remove_dir_all(root).expect("config temp dir should be removed");
    }

    #[test]
    fn unknown_keys_are_tolerated_by_serde_default() {
        let root = temp_dir("unknown-keys");
        fs::write(
            root.join(".deslop.toml"),
            "go_semantic_experimental = true\nfuture_unknown_setting = 42\n",
        )
        .expect("config file should be written");

        // TOML serde `deny_unknown_fields` is NOT set, so unknown keys should be ignored.
        let result = load_repository_config(&root);
        // If the project later adds `deny_unknown_fields`, this test should be updated.
        assert!(
            result.is_ok(),
            "unknown keys should be silently ignored: {result:?}"
        );

        fs::remove_dir_all(root).expect("config temp dir should be removed");
    }
}
