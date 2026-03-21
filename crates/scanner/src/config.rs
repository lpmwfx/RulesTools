use std::collections::HashMap;
use std::path::Path;

/// Unified scan configuration — parsed from `proj/rulestools.toml`.
#[derive(Debug, Clone)]
pub struct Config {
    pub languages: Vec<String>,
    pub topology: String,
    pub deny: bool,
    pub exclude: Vec<String>,
    checks: HashMap<String, bool>,
    params: HashMap<String, toml::Value>,
}

impl Config {
    /// Parse config from TOML string content.
    pub fn parse(content: &str) -> Self {
        let table: toml::Table = content.parse().unwrap_or_default();
        let mut cfg = Self::default();

        // Languages: [scan].languages > [project].languages > top-level languages
        if let Some(langs) = Self::extract_string_array(&table, &["scan", "languages"])
            .or_else(|| Self::extract_string_array(&table, &["project", "languages"]))
            .or_else(|| {
                table.get("languages").and_then(|v| {
                    v.as_array().map(|a| {
                        a.iter().filter_map(|x| x.as_str().map(String::from)).collect()
                    })
                })
            })
        {
            cfg.languages = langs;
        }

        // Exclude patterns: [scan].exclude
        if let Some(excludes) = Self::extract_string_array(&table, &["scan", "exclude"]) {
            cfg.exclude = excludes;
        }

        // Topology: [project].topology
        if let Some(topo) = table
            .get("project")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("topology"))
            .and_then(|v| v.as_str())
        {
            cfg.topology = topo.to_string();
        }

        // [checks] section — new unified format
        if let Some(checks_table) = table.get("checks").and_then(|v| v.as_table()) {
            if let Some(d) = checks_table.get("deny").and_then(|v| v.as_bool()) {
                cfg.deny = d;
            }
            for (key, val) in checks_table {
                if key == "deny" {
                    continue;
                }
                if let Some(enabled) = val.as_bool() {
                    cfg.checks.insert(key.clone(), enabled);
                }
            }
        }

        // Legacy [rustscanners] — map to unified keys
        if let Some(rs_table) = table.get("rustscanners").and_then(|v| v.as_table()) {
            Self::map_legacy_section(&mut cfg.checks, &mut cfg.params, rs_table, "rust");
        }

        // Legacy [slintscanners] — map to unified keys
        if let Some(sl_table) = table.get("slintscanners").and_then(|v| v.as_table()) {
            Self::map_legacy_section(&mut cfg.checks, &mut cfg.params, sl_table, "slint");
        }

        cfg
    }

    /// Load config from a project directory (reads `proj/rulestools.toml`).
    pub fn load(project_root: &Path) -> Self {
        let config_path = project_root.join("proj").join("rulestools.toml");
        match std::fs::read_to_string(&config_path) {
            Ok(content) => Self::parse(&content),
            Err(_) => Self::default(),
        }
    }

    /// Check if a specific check is enabled. Defaults to true if not configured.
    pub fn is_enabled(&self, check_id: &str) -> bool {
        self.checks.get(check_id).copied().unwrap_or(true)
    }

    /// Get a parameter value for a check, with a default fallback.
    pub fn param_i64(&self, key: &str, default: i64) -> i64 {
        self.params
            .get(key)
            .and_then(|v| v.as_integer())
            .unwrap_or(default)
    }

    /// Get a parameter string value.
    pub fn param_str(&self, key: &str, default: &str) -> String {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| default.to_string())
    }

    fn extract_string_array(table: &toml::Table, keys: &[&str]) -> Option<Vec<String>> {
        let mut current: &toml::Value = table.get(*keys.first()?)?;
        for key in &keys[1..] {
            current = current.as_table()?.get(*key)?;
        }
        current.as_array().map(|a| {
            a.iter().filter_map(|x| x.as_str().map(String::from)).collect()
        })
    }

    fn map_legacy_section(
        checks: &mut HashMap<String, bool>,
        params: &mut HashMap<String, toml::Value>,
        table: &toml::Table,
        prefix: &str,
    ) {
        for (key, val) in table {
            if let Some(enabled) = val.as_bool() {
                let unified_key = format!("{prefix}/{key}");
                checks.entry(unified_key).or_insert(enabled);
            } else {
                let param_key = format!("{prefix}/{key}");
                params.insert(param_key, val.clone());
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
            topology: String::from("flat"),
            deny: false,
            exclude: Vec::new(),
            checks: HashMap::new(),
            params: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_enabled() {
        let cfg = Config::default();
        assert!(cfg.is_enabled("rust/magic_numbers"));
        assert!(cfg.is_enabled("slint/tokens"));
        assert!(cfg.is_enabled("anything/unknown"));
    }

    #[test]
    fn parse_checks_section() {
        let content = r#"
[checks]
deny = true
"rust/magic_numbers" = true
"slint/tokens" = false
"#;
        let cfg = Config::parse(content);
        assert!(cfg.deny);
        assert!(cfg.is_enabled("rust/magic_numbers"));
        assert!(!cfg.is_enabled("slint/tokens"));
    }

    #[test]
    fn parse_legacy_rustscanners() {
        let content = r#"
[rustscanners]
magic_numbers = false
child_module_warn_at = 100
"#;
        let cfg = Config::parse(content);
        assert!(!cfg.is_enabled("rust/magic_numbers"));
        assert_eq!(cfg.param_i64("rust/child_module_warn_at", 50), 100);
    }

    #[test]
    fn parse_legacy_slintscanners() {
        let content = r#"
[slintscanners]
tokens = true
architecture = false
"#;
        let cfg = Config::parse(content);
        assert!(cfg.is_enabled("slint/tokens"));
        assert!(!cfg.is_enabled("slint/architecture"));
    }

    #[test]
    fn parse_topology() {
        let content = r#"
[project]
topology = "workspace"
"#;
        let cfg = Config::parse(content);
        assert_eq!(cfg.topology, "workspace");
    }

    #[test]
    fn parse_languages_scan_section() {
        let content = r#"
[scan]
languages = ["rust", "slint"]
"#;
        let cfg = Config::parse(content);
        assert_eq!(cfg.languages, vec!["rust", "slint"]);
    }

    #[test]
    fn parse_languages_fallback() {
        let content = r#"
[project]
languages = ["python"]
"#;
        let cfg = Config::parse(content);
        assert_eq!(cfg.languages, vec!["python"]);
    }

    #[test]
    fn parse_top_level_languages() {
        let content = r#"
languages = ["rust"]
"#;
        let cfg = Config::parse(content);
        assert_eq!(cfg.languages, vec!["rust"]);
    }

    #[test]
    fn empty_content() {
        let cfg = Config::parse("");
        assert!(cfg.languages.is_empty());
        assert_eq!(cfg.topology, "flat");
        assert!(!cfg.deny);
        assert!(cfg.is_enabled("any/check"));
        assert!(cfg.exclude.is_empty());
    }

    #[test]
    fn parse_exclude_patterns() {
        let content = r#"
[scan]
exclude = ["ui/tokens/*.slint", "**/generated/**"]
"#;
        let cfg = Config::parse(content);
        assert_eq!(cfg.exclude, vec!["ui/tokens/*.slint", "**/generated/**"]);
    }

    #[test]
    fn parse_exclude_with_languages() {
        let content = r#"
[scan]
languages = ["rust", "slint"]
exclude = ["ui/app-window.slint"]
"#;
        let cfg = Config::parse(content);
        assert_eq!(cfg.languages, vec!["rust", "slint"]);
        assert_eq!(cfg.exclude, vec!["ui/app-window.slint"]);
    }
}
