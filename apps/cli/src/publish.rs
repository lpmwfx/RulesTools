use std::path::Path;

// --- Config structs ---

/// struct `PublishConfig`.
pub struct PublishConfig {
    pub targets: Vec<String>,
    pub platforms: Vec<String>,
    pub license: String,
    pub author: String,
    pub github: Option<GithubTarget>,
    pub forgejo: Option<ForgejoTarget>,
    pub repo: Option<RepoConfig>,
}

/// struct `GithubTarget`.
pub struct GithubTarget {
    pub repo: String,
}

/// struct `ForgejoTarget`.
pub struct ForgejoTarget {
    pub repo: String,
    pub api_url: String,
}

/// struct `RepoConfig`.
pub struct RepoConfig {
    pub path: String,
    pub remote: String,
    pub include_files: Vec<String>,
    pub include_dirs: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// Dirs that are ALWAYS excluded — cannot be overridden.
const HARDCODED_EXCLUDE_DIRS: &[&str] = &["proj", "doc", "man", "target", ".claude", ".git"];

/// Patterns that are ALWAYS excluded — cannot be overridden.
const HARDCODED_EXCLUDE_PATTERNS: &[&str] = &[
    "*.secret", "*.key", ".env*", "TODO", "FIXES", "ISSUES",
];

// --- Config parsing ---

impl PublishConfig {
    /// fn `load`.
    pub fn load(root: &Path) -> Self {
        let toml_path = root.join("proj").join("rulestools.toml");
        let table = if toml_path.exists() {
            std::fs::read_to_string(&toml_path)
                .ok()
                .and_then(|s| s.parse::<toml::Table>().ok())
                .unwrap_or_default()
        } else {
            toml::Table::new()
        };

        let publish = table.get("publish").and_then(|v| v.as_table());

        let targets = publish
            .and_then(|p| p.get("targets"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let platforms = publish
            .and_then(|p| p.get("platforms"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(|| vec![host_triple().to_string()]);

        let license = publish
            .and_then(|p| p.get("license"))
            .and_then(|v| v.as_str())
            .unwrap_or("EUPL-1.2")
            .to_string();

        let author = publish
            .and_then(|p| p.get("author"))
            .and_then(|v| v.as_str())
            .unwrap_or("TwistedBrain")
            .to_string();

        let github = publish
            .and_then(|p| p.get("github"))
            .and_then(|v| v.as_table())
            .and_then(|t| {
                t.get("repo").and_then(|v| v.as_str()).map(|repo| GithubTarget {
                    repo: repo.to_string(),
                })
            });

        let forgejo = publish
            .and_then(|p| p.get("forgejo"))
            .and_then(|v| v.as_table())
            .and_then(|t| {
                let repo = t.get("repo").and_then(|v| v.as_str())?;
                let api_url = t.get("api_url").and_then(|v| v.as_str())?;
                Some(ForgejoTarget {
                    repo: repo.to_string(),
                    api_url: api_url.to_string(),
                })
            });

        let repo = publish
            .and_then(|p| p.get("repo"))
            .and_then(|v| v.as_table())
            .map(|t| {
                let path = t.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let remote = t.get("remote").and_then(|v| v.as_str()).unwrap_or("").to_string();

                let include = t.get("include").and_then(|v| v.as_table());
                let exclude = t.get("exclude").and_then(|v| v.as_table());

                let include_files = include
                    .and_then(|i| i.get("files"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let include_dirs = include
                    .and_then(|i| i.get("dirs"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let mut exclude_dirs: Vec<String> = exclude
                    .and_then(|e| e.get("dirs"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                // Merge hardcoded excludes
                for hc in HARDCODED_EXCLUDE_DIRS {
                    let s = hc.to_string();
                    if !exclude_dirs.contains(&s) {
                        exclude_dirs.push(s);
                    }
                }

                let mut exclude_patterns: Vec<String> = exclude
                    .and_then(|e| e.get("patterns"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                for hc in HARDCODED_EXCLUDE_PATTERNS {
                    let s = hc.to_string();
                    if !exclude_patterns.contains(&s) {
                        exclude_patterns.push(s);
                    }
                }

                RepoConfig {
                    path,
                    remote,
                    include_files,
                    include_dirs,
                    exclude_dirs,
                    exclude_patterns,
                }
            });

        PublishConfig {
            targets,
            platforms,
            license,
            author,
            github,
            forgejo,
            repo,
        }
    }
}

// --- Helpers ---

fn host_triple() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-windows"
        } else {
            "aarch64-windows"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-macos"
        } else {
            "x86_64-macos"
        }
    } else {
        "x86_64-linux"
    }
}

/// Read version from Cargo.toml (workspace or package).
fn read_version_cargo(root: &Path) -> Option<String> {
    let cargo_path = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path).ok()?;
    let table: toml::Table = content.parse().ok()?;

    // Try [workspace.package].version first
    if let Some(ver) = table
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        return Some(ver.to_string());
    }

    // Then [package].version
    if let Some(ver) = table
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        return Some(ver.to_string());
    }

    None
}

/// Read version from package.json.
fn read_version_package_json(root: &Path) -> Option<String> {
    let pkg_path = root.join("package.json");
    let content = std::fs::read_to_string(&pkg_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version").and_then(|v| v.as_str()).map(String::from)
}

/// Read project version — tries Cargo.toml then package.json.
fn read_version(root: &Path) -> Option<String> {
    read_version_cargo(root).or_else(|| read_version_package_json(root))
}

/// Read project name from Cargo.toml or directory.
fn read_project_name(root: &Path) -> String {
    let cargo_path = root.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_path) {
        if let Ok(table) = content.parse::<toml::Table>() {
            // Try [workspace.package].name, then [package].name
            if let Some(name) = table
                .get("workspace")
                .and_then(|w| w.get("package"))
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
            {
                return name.to_string();
            }
            if let Some(name) = table
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
            {
                return name.to_string();
            }
        }
    }
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

/// Run a shell command and return stdout.
fn run_cmd(cmd: &str, args: &[&str], cwd: &Path) -> Result<String, String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Cannot run {cmd}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        Err(if stderr.is_empty() { stdout } else { stderr })
    } else {
        Ok(stdout)
    }
}

/// Check if git tree is clean.
fn git_is_clean(root: &Path) -> Result<bool, String> {
    let out = run_cmd("git", &["status", "--porcelain"], root)?;
    Ok(out.is_empty())
}

/// Get last tag.
fn git_last_tag(root: &Path) -> Option<String> {
    run_cmd("git", &["describe", "--tags", "--abbrev=0"], root).ok()
}

/// Get changelog since tag (or all if no tag).
fn git_changelog(root: &Path, since_tag: Option<&str>) -> String {
    match since_tag {
        Some(tag) => {
            let range = format!("{tag}..HEAD");
            run_cmd("git", &["log", "--oneline", &range], root).unwrap_or_default()
        }
        None => run_cmd("git", &["log", "--oneline", "-20"], root).unwrap_or_default(),
    }
}

/// Count commits since tag.
fn git_commit_count_since(root: &Path, tag: &str) -> usize {
    run_cmd("git", &["rev-list", "--count", &format!("{tag}..HEAD")], root)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Check if a pattern matches a filename (simple glob: *.ext, .env*).
fn pattern_matches(pattern: &str, filename: &str) -> bool {
    if pattern.starts_with('*') {
        // *.ext
        filename.ends_with(&pattern[1..])
    } else if pattern.ends_with('*') {
        // .env*
        filename.starts_with(&pattern[..pattern.len() - 1])
    } else {
        filename == pattern
    }
}

/// Check if a path should be excluded based on RepoConfig.
fn is_excluded(rel_path: &Path, repo_cfg: &RepoConfig) -> bool {
    let path_str = rel_path.to_string_lossy();

    // Check dir excludes (hardcoded + config)
    for dir in &repo_cfg.exclude_dirs {
        if path_str.starts_with(&format!("{dir}/")) || path_str.starts_with(&format!("{dir}\\")) || &*path_str == dir {
            return true;
        }
    }

    // Check pattern excludes
    if let Some(filename) = rel_path.file_name().and_then(|n| n.to_str()) {
        for pattern in &repo_cfg.exclude_patterns {
            if pattern_matches(pattern, filename) {
                return true;
            }
        }
    }

    false
}

// --- Publish Plan ---

/// struct `PublishPlan`.
pub struct PublishPlan {
    pub name: String,
    pub version: String,
    pub git_clean: bool,
    pub last_tag: Option<String>,
    pub commits_since_tag: usize,
    pub changelog: String,
    pub targets: Vec<TargetPlan>,
    pub scanner_clean: Option<bool>,
}

/// struct `TargetPlan`.
pub struct TargetPlan {
    pub name: String,
    pub detail: String,
}

/// fn `publish_plan`.
pub fn publish_plan(root: &Path, format: &str) -> Result<String, String> {
    let cfg = PublishConfig::load(root);
    let name = read_project_name(root);
    let version = read_version(root).unwrap_or_else(|| "0.0.0".to_string());
    let git_clean = git_is_clean(root).unwrap_or(false);
    let last_tag = git_last_tag(root);
    let commits_since = last_tag.as_ref().map(|t| git_commit_count_since(root, t)).unwrap_or(0);
    let changelog = git_changelog(root, last_tag.as_deref());

    // Run scanner check
    let scanner_clean = run_cmd("rulestools", &["check", &root.to_string_lossy()], root)
        .map(|_| true)
        .ok();

    if cfg.targets.is_empty() {
        return Err("No publish targets configured.\nAdd [publish] section to proj/rulestools.toml".to_string());
    }

    let mut targets = Vec::new();
    for target_name in &cfg.targets {
        let detail = match target_name.as_str() {
            "github" => {
                if let Some(ref gh) = cfg.github {
                    let tag_info = last_tag.as_ref()
                        .map(|t| format!("{commits_since} commits since {t}"))
                        .unwrap_or_else(|| "no previous tag".to_string());
                    format!("{} (v{version}, {tag_info})", gh.repo)
                } else {
                    "[publish.github] not configured".to_string()
                }
            }
            "forgejo" => {
                if let Some(ref fg) = cfg.forgejo {
                    format!("{} (v{version})", fg.repo)
                } else {
                    "[publish.forgejo] not configured".to_string()
                }
            }
            "archive" => {
                let triple = cfg.platforms.first().map(|s| s.as_str()).unwrap_or(host_triple());
                let ext = if triple.contains("windows") { "zip" } else { "tar.gz" };
                format!("target/publish/{name}-{version}-{triple}.{ext}")
            }
            other => format!("unknown target: {other}"),
        };
        targets.push(TargetPlan { name: target_name.clone(), detail });
    }

    let plan = PublishPlan {
        name: name.clone(),
        version: version.clone(),
        git_clean,
        last_tag,
        commits_since_tag: commits_since,
        changelog,
        targets,
        scanner_clean,
    };

    if format == "json" {
        Ok(format_plan_json(&plan))
    } else {
        Ok(format_plan_text(&plan))
    }
}

fn format_plan_text(plan: &PublishPlan) -> String {
    let mut out = format!("Publish plan for {} v{}\n\n", plan.name, plan.version);

    out.push_str("  Pre-checks:\n");
    let scanner_icon = match plan.scanner_clean {
        Some(true) => "ok",
        Some(false) => "FAIL",
        None => "skip",
    };
    out.push_str(&format!("    [{scanner_icon}] Scanner\n"));
    let git_icon = if plan.git_clean { "ok" } else { "FAIL" };
    out.push_str(&format!("    [{git_icon}] Git tree clean\n"));
    out.push('\n');

    out.push_str("  Targets:\n");
    for t in &plan.targets {
        out.push_str(&format!("    {:<10} -> {}\n", t.name, t.detail));
    }

    if !plan.changelog.is_empty() {
        out.push_str("\n  Changelog:\n");
        for line in plan.changelog.lines().take(15) {
            out.push_str(&format!("    {line}\n"));
        }
    }

    out
}

fn format_plan_json(plan: &PublishPlan) -> String {
    let targets: Vec<serde_json::Value> = plan.targets.iter().map(|t| {
        serde_json::json!({
            "name": t.name,
            "detail": t.detail,
        })
    }).collect();

    let json = serde_json::json!({
        "name": plan.name,
        "version": plan.version,
        "git_clean": plan.git_clean,
        "last_tag": plan.last_tag,
        "commits_since_tag": plan.commits_since_tag,
        "scanner_clean": plan.scanner_clean,
        "targets": targets,
        "changelog": plan.changelog,
    });
    serde_json::to_string_pretty(&json).unwrap_or_default()
}

// --- Publish Status ---

/// fn `publish_status`.
pub fn publish_status(root: &Path, format: &str) -> Result<String, String> {
    let cfg = PublishConfig::load(root);

    if cfg.targets.is_empty() {
        return Err("No publish targets configured".to_string());
    }

    let mut entries: Vec<(String, String, String, String)> = Vec::new();

    for target_name in &cfg.targets {
        match target_name.as_str() {
            "github" => {
                if let Some(ref gh) = cfg.github {
                    match fetch_github_latest_release(&gh.repo) {
                        Ok((ver, date, url)) => entries.push(("github".into(), ver, date, url)),
                        Err(e) => entries.push(("github".into(), "none".into(), "".into(), e)),
                    }
                } else {
                    entries.push(("github".into(), "not configured".into(), "".into(), "".into()));
                }
            }
            "forgejo" => {
                if let Some(ref fg) = cfg.forgejo {
                    match fetch_forgejo_latest_release(&fg.api_url) {
                        Ok((ver, date, url)) => entries.push(("forgejo".into(), ver, date, url)),
                        Err(e) => entries.push(("forgejo".into(), "none".into(), "".into(), e)),
                    }
                } else {
                    entries.push(("forgejo".into(), "not configured".into(), "".into(), "".into()));
                }
            }
            "archive" => {
                let name = read_project_name(root);
                let version = read_version(root).unwrap_or_else(|| "0.0.0".to_string());
                let triple = cfg.platforms.first().map(|s| s.as_str()).unwrap_or(host_triple());
                let ext = if triple.contains("windows") { "zip" } else { "tar.gz" };
                let archive_path = root.join("target").join("publish").join(format!("{name}-{version}-{triple}.{ext}"));
                if archive_path.exists() {
                    entries.push(("archive".into(), version, "local".into(), archive_path.to_string_lossy().to_string()));
                } else {
                    entries.push(("archive".into(), "not built".into(), "".into(), "".into()));
                }
            }
            other => {
                entries.push((other.to_string(), "unknown target".into(), "".into(), "".into()));
            }
        }
    }

    if format == "json" {
        let arr: Vec<serde_json::Value> = entries.iter().map(|(target, ver, date, url)| {
            serde_json::json!({
                "target": target,
                "version": ver,
                "date": date,
                "url": url,
            })
        }).collect();
        Ok(serde_json::to_string_pretty(&arr).unwrap_or_default())
    } else {
        let mut out = format!("{:<12} {:<12} {:<12} {}\n", "TARGET", "VERSION", "DATE", "URL");
        out.push_str(&format!("{}\n", "-".repeat(60)));
        for (target, ver, date, url) in &entries {
            out.push_str(&format!("{:<12} {:<12} {:<12} {}\n", target, ver, date, url));
        }
        Ok(out)
    }
}

fn fetch_github_latest_release(repo: &str) -> Result<(String, String, String), String> {
    let token = std::env::var("GITHUB_TOKEN").unwrap_or_default();
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");

    let req = ureq::get(&url).set("User-Agent", "rulestools");
    let req = if !token.is_empty() {
        req.set("Authorization", &format!("Bearer {token}"))
    } else {
        req
    };

    let resp = req.call().map_err(|e| format!("No releases: {e}"))?;
    let body = resp.into_string().map_err(|e| format!("{e}"))?;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();

    let tag = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
    let date = json.get("published_at").and_then(|v| v.as_str()).unwrap_or("?")
        .chars().take(10).collect::<String>();
    let url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();

    Ok((tag, date, url))
}

fn fetch_forgejo_latest_release(api_url: &str) -> Result<(String, String, String), String> {
    let token = std::env::var("FORGEJO_TOKEN").unwrap_or_default();
    let url = format!("{api_url}/releases?limit=1");

    let req = ureq::get(&url);
    let req = if !token.is_empty() {
        req.set("Authorization", &format!("token {token}"))
    } else {
        req
    };

    let resp = req.call().map_err(|e| format!("No releases: {e}"))?;
    let body = resp.into_string().map_err(|e| format!("{e}"))?;
    let arr: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();

    let release = arr.first().ok_or_else(|| "No releases found".to_string())?;
    let tag = release.get("tag_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
    let date = release.get("created_at").and_then(|v| v.as_str()).unwrap_or("?")
        .chars().take(10).collect::<String>();
    let url = release.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();

    Ok((tag, date, url))
}

// --- Publish Run ---

/// fn `publish_run`.
pub fn publish_run(root: &Path, target: &str, preview: bool) -> Result<String, String> {
    let cfg = PublishConfig::load(root);
    let name = read_project_name(root);
    let version = read_version(root).ok_or("Cannot read version from Cargo.toml or package.json")?;

    if !cfg.targets.contains(&target.to_string()) {
        return Err(format!("Target '{target}' not in configured targets: {:?}", cfg.targets));
    }

    // Pre-publish checks
    let mut checks_ok = true;
    let mut check_report = String::new();

    // 1. Scanner check
    match run_cmd("rulestools", &["check", &root.to_string_lossy()], root) {
        Ok(_) => check_report.push_str("  [ok] Scanner clean\n"),
        Err(e) => {
            check_report.push_str(&format!("  [FAIL] Scanner: {e}\n"));
            checks_ok = false;
        }
    }

    // 2. Tests (skip for non-Rust)
    let has_cargo = root.join("Cargo.toml").exists();
    if has_cargo {
        match run_cmd("cargo", &["test", "--quiet"], root) {
            Ok(_) => check_report.push_str("  [ok] Tests pass\n"),
            Err(e) => {
                check_report.push_str(&format!("  [FAIL] Tests: {e}\n"));
                checks_ok = false;
            }
        }
    }

    // 3. Git clean
    match git_is_clean(root) {
        Ok(true) => check_report.push_str("  [ok] Git tree clean\n"),
        Ok(false) => {
            check_report.push_str("  [FAIL] Git tree has uncommitted changes\n");
            checks_ok = false;
        }
        Err(e) => {
            check_report.push_str(&format!("  [FAIL] Git: {e}\n"));
            checks_ok = false;
        }
    }

    // 4. Version set
    check_report.push_str(&format!("  [ok] Version: {version}\n"));

    // 5. Token available
    match target {
        "github" => {
            if std::env::var("GITHUB_TOKEN").is_err() {
                check_report.push_str("  [FAIL] GITHUB_TOKEN not set\n");
                checks_ok = false;
            } else {
                check_report.push_str("  [ok] GITHUB_TOKEN set\n");
            }
        }
        "forgejo" => {
            if std::env::var("FORGEJO_TOKEN").is_err() {
                check_report.push_str("  [FAIL] FORGEJO_TOKEN not set\n");
                checks_ok = false;
            } else {
                check_report.push_str("  [ok] FORGEJO_TOKEN set\n");
            }
        }
        "archive" => { /* no token needed */ }
        _ => {}
    }

    if preview {
        return Ok(format!("Publish preview for {name} v{version} -> {target}\n\nPre-checks:\n{check_report}\nPreview mode — no changes made."));
    }

    if !checks_ok {
        return Err(format!("Pre-publish checks failed:\n{check_report}"));
    }

    match target {
        "github" => publish_to_github(root, &cfg, &name, &version),
        "forgejo" => publish_to_forgejo(root, &cfg, &name, &version),
        "archive" => publish_to_archive(root, &cfg, &name, &version),
        _ => Err(format!("Unknown target: {target}")),
    }
}

fn publish_to_github(root: &Path, cfg: &PublishConfig, name: &str, version: &str) -> Result<String, String> {
    let gh = cfg.github.as_ref().ok_or("[publish.github] not configured")?;
    let token = std::env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN not set")?;

    // Build (Rust projects)
    if root.join("Cargo.toml").exists() {
        run_cmd("cargo", &["build", "--release"], root)?;
    }

    // Generate changelog
    let last_tag = git_last_tag(root);
    let changelog = git_changelog(root, last_tag.as_deref());

    // Tag
    let tag = format!("v{version}");
    run_cmd("git", &["tag", &tag], root)?;
    run_cmd("git", &["push", "--tags"], root)?;

    // Create release
    let payload = serde_json::json!({
        "tag_name": tag,
        "name": tag,
        "body": changelog,
    });

    let url = format!("https://api.github.com/repos/{}/releases", gh.repo);
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "rulestools")
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("GitHub API error: {e}"))?;

    let body = resp.into_string().unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    let release_url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or("?");

    Ok(format!("Published {name} {tag} to GitHub\nRelease: {release_url}"))
}

fn publish_to_forgejo(root: &Path, cfg: &PublishConfig, name: &str, version: &str) -> Result<String, String> {
    let fg = cfg.forgejo.as_ref().ok_or("[publish.forgejo] not configured")?;
    let token = std::env::var("FORGEJO_TOKEN").map_err(|_| "FORGEJO_TOKEN not set")?;

    // Build (Rust projects)
    if root.join("Cargo.toml").exists() {
        run_cmd("cargo", &["build", "--release"], root)?;
    }

    // Generate changelog
    let last_tag = git_last_tag(root);
    let changelog = git_changelog(root, last_tag.as_deref());

    // Tag
    let tag = format!("v{version}");
    run_cmd("git", &["tag", &tag], root)?;
    run_cmd("git", &["push", "--tags"], root)?;

    // Create release
    let payload = serde_json::json!({
        "tag_name": tag,
        "name": tag,
        "body": changelog,
    });

    let url = format!("{}/releases", fg.api_url);
    let resp = ureq::post(&url)
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("Forgejo API error: {e}"))?;

    let body = resp.into_string().unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    let release_url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or("?");

    Ok(format!("Published {name} {tag} to Forgejo\nRelease: {release_url}"))
}

fn publish_to_archive(root: &Path, cfg: &PublishConfig, name: &str, version: &str) -> Result<String, String> {
    // Build
    if root.join("Cargo.toml").exists() {
        run_cmd("cargo", &["build", "--release"], root)?;
    }

    let triple = cfg.platforms.first().map(|s| s.as_str()).unwrap_or(host_triple());
    let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };

    let publish_dir = root.join("target").join("publish");
    std::fs::create_dir_all(&publish_dir).map_err(|e| format!("Cannot create target/publish: {e}"))?;

    let archive_name = format!("{name}-{version}-{triple}.{ext}");
    let archive_path = publish_dir.join(&archive_name);

    // Collect files for archive
    let staging = publish_dir.join("_staging");
    if staging.exists() {
        std::fs::remove_dir_all(&staging).ok();
    }
    std::fs::create_dir_all(&staging).map_err(|e| format!("Cannot create staging dir: {e}"))?;

    // Copy binary
    let binary_name = if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    let binary_src = root.join("target").join("release").join(&binary_name);
    if binary_src.exists() {
        std::fs::copy(&binary_src, staging.join(&binary_name))
            .map_err(|e| format!("Cannot copy binary: {e}"))?;
    }

    // Copy README and LICENSE if they exist
    for file in &["README.md", "LICENSE", "LICENSE.md"] {
        let src = root.join(file);
        if src.exists() {
            std::fs::copy(&src, staging.join(file)).ok();
        }
    }

    // Create archive
    if cfg!(target_os = "windows") {
        // Use PowerShell Compress-Archive on Windows
        let staging_str = staging.to_string_lossy().to_string();
        let archive_str = archive_path.to_string_lossy().to_string();
        // Remove existing archive
        if archive_path.exists() {
            std::fs::remove_file(&archive_path).ok();
        }
        run_cmd(
            "powershell",
            &["-NoProfile", "-Command",
              &format!("Compress-Archive -Path '{staging_str}\\*' -DestinationPath '{archive_str}'")],
            root,
        )?;
    } else {
        let archive_str = archive_path.to_string_lossy().to_string();
        run_cmd(
            "tar",
            &["czf", &archive_str, "-C", &staging.to_string_lossy(), "."],
            root,
        )?;
    }

    // Clean up staging
    std::fs::remove_dir_all(&staging).ok();

    Ok(format!("Archive created: {}", archive_path.display()))
}

// --- Publish Init ---

/// fn `publish_init`.
pub fn publish_init(root: &Path, remote: &str) -> Result<String, String> {
    let name = read_project_name(root);
    let pub_path = root.join("..").join(format!("{name}-pub"));
    let pub_path = pub_path.canonicalize().unwrap_or(pub_path);

    if pub_path.exists() {
        return Err(format!("Pub-repo already exists: {}", pub_path.display()));
    }

    // Create pub-repo
    std::fs::create_dir_all(&pub_path).map_err(|e| format!("Cannot create pub-repo: {e}"))?;

    // Init git
    run_cmd("git", &["init"], &pub_path)?;
    run_cmd("git", &["remote", "add", "origin", remote], &pub_path)?;

    // Calculate relative path
    let rel_path = format!("../{name}-pub");

    // Append config to rulestools.toml
    let toml_path = root.join("proj").join("rulestools.toml");
    let mut content = std::fs::read_to_string(&toml_path).unwrap_or_default();

    content.push_str(&format!(r#"
[publish]
targets = ["github"]
license = "EUPL-1.2"
author = "TwistedBrain"

[publish.repo]
path = "{rel_path}"
remote = "{remote}"

[publish.repo.include]
files = ["Cargo.toml", "build.rs", "LICENSE", ".gitignore", "README.md"]
dirs = ["src", "crates", "apps"]

[publish.repo.exclude]
dirs = ["proj", "doc", "man", "target", ".claude"]
patterns = ["*.secret", "*.key", ".env*"]
"#));

    std::fs::write(&toml_path, content).map_err(|e| format!("Cannot write config: {e}"))?;

    // Run initial sync
    let sync_result = publish_sync(root, true)?;

    Ok(format!("Pub-repo initialized: {}\nRemote: {remote}\nConfig added to proj/rulestools.toml\n\n{sync_result}", pub_path.display()))
}

// --- Publish Sync ---

/// fn `publish_sync`.
pub fn publish_sync(root: &Path, preview: bool) -> Result<String, String> {
    let cfg = PublishConfig::load(root);
    let repo_cfg = cfg.repo.as_ref().ok_or("[publish.repo] not configured in proj/rulestools.toml")?;

    if repo_cfg.path.is_empty() {
        return Err("[publish.repo].path is empty".to_string());
    }

    let pub_path = root.join(&repo_cfg.path);
    if !pub_path.exists() {
        return Err(format!("Pub-repo not found: {}\nRun `rulestools publish init` first", pub_path.display()));
    }

    let mut new_files: Vec<String> = Vec::new();
    let mut updated_files: Vec<String> = Vec::new();
    let mut unchanged_files: Vec<String> = Vec::new();
    let mut skipped_excluded: Vec<String> = Vec::new();

    // Sync individual files
    for file in &repo_cfg.include_files {
        let rel = Path::new(file);
        if is_excluded(rel, repo_cfg) {
            skipped_excluded.push(file.clone());
            continue;
        }
        let src = root.join(file);
        let dst = pub_path.join(file);
        if src.exists() {
            match sync_file(&src, &dst, preview) {
                SyncResult::New => new_files.push(file.clone()),
                SyncResult::Updated => updated_files.push(file.clone()),
                SyncResult::Unchanged => unchanged_files.push(file.clone()),
            }
        }
    }

    // Sync directories
    for dir in &repo_cfg.include_dirs {
        let src_dir = root.join(dir);
        if !src_dir.exists() {
            continue;
        }
        sync_dir_recursive(
            root, &pub_path, &src_dir, dir, repo_cfg, preview,
            &mut new_files, &mut updated_files, &mut unchanged_files, &mut skipped_excluded,
        );
    }

    // Commit in pub-repo (non-preview)
    if !preview && (!new_files.is_empty() || !updated_files.is_empty()) {
        let version = read_version(root).unwrap_or_else(|| "dev".to_string());
        run_cmd("git", &["add", "-A"], &pub_path).ok();
        run_cmd("git", &["commit", "-m", &format!("sync v{version}")], &pub_path).ok();
    }

    let mut out = String::new();
    let label = if preview { "Sync preview" } else { "Synced" };
    out.push_str(&format!("{label}:\n"));
    if !new_files.is_empty() {
        out.push_str(&format!("  New:       {} files\n", new_files.len()));
        for f in &new_files {
            out.push_str(&format!("    + {f}\n"));
        }
    }
    if !updated_files.is_empty() {
        out.push_str(&format!("  Updated:   {} files\n", updated_files.len()));
        for f in &updated_files {
            out.push_str(&format!("    ~ {f}\n"));
        }
    }
    if !unchanged_files.is_empty() {
        out.push_str(&format!("  Unchanged: {} files\n", unchanged_files.len()));
    }
    if !skipped_excluded.is_empty() {
        out.push_str(&format!("  Excluded:  {} items\n", skipped_excluded.len()));
    }
    if new_files.is_empty() && updated_files.is_empty() {
        out.push_str("  Nothing to sync — pub-repo is up-to-date\n");
    }

    Ok(out)
}

enum SyncResult {
    New,
    Updated,
    Unchanged,
}

fn sync_file(src: &Path, dst: &Path, preview: bool) -> SyncResult {
    let src_content = std::fs::read(src).unwrap_or_default();

    if !dst.exists() {
        if !preview {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(dst, &src_content).ok();
        }
        return SyncResult::New;
    }

    let dst_content = std::fs::read(dst).unwrap_or_default();
    if src_content == dst_content {
        SyncResult::Unchanged
    } else {
        if !preview {
            std::fs::write(dst, &src_content).ok();
        }
        SyncResult::Updated
    }
}

fn sync_dir_recursive(
    root: &Path,
    pub_root: &Path,
    src_dir: &Path,
    rel_prefix: &str,
    repo_cfg: &RepoConfig,
    preview: bool,
    new_files: &mut Vec<String>,
    updated_files: &mut Vec<String>,
    unchanged_files: &mut Vec<String>,
    skipped: &mut Vec<String>,
) {
    let entries = match std::fs::read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let rel = format!("{rel_prefix}/{file_name}");
        let rel_path = Path::new(&rel);

        if is_excluded(rel_path, repo_cfg) {
            skipped.push(rel.clone());
            continue;
        }

        if entry_path.is_dir() {
            sync_dir_recursive(
                root, pub_root, &entry_path, &rel, repo_cfg, preview,
                new_files, updated_files, unchanged_files, skipped,
            );
        } else {
            let dst = pub_root.join(&rel);
            match sync_file(&entry_path, &dst, preview) {
                SyncResult::New => new_files.push(rel),
                SyncResult::Updated => updated_files.push(rel),
                SyncResult::Unchanged => unchanged_files.push(rel),
            }
        }
    }
}

// --- Publish Check ---

/// fn `publish_check`.
pub fn publish_check(root: &Path) -> Result<String, String> {
    let cfg = PublishConfig::load(root);
    let repo_cfg = cfg.repo.as_ref().ok_or("[publish.repo] not configured")?;

    if repo_cfg.path.is_empty() {
        return Err("[publish.repo].path is empty".to_string());
    }

    let pub_path = root.join(&repo_cfg.path);
    if !pub_path.exists() {
        return Err(format!("Pub-repo not found: {}", pub_path.display()));
    }

    let mut leaked: Vec<String> = Vec::new();
    let mut out_of_sync: Vec<String> = Vec::new();

    // Walk pub-repo and check for leaks
    walk_and_check(&pub_path, &pub_path, repo_cfg, root, &mut leaked, &mut out_of_sync);

    let mut out = String::new();
    if leaked.is_empty() && out_of_sync.is_empty() {
        out.push_str("CLEAN — pub-repo passes all checks\n");
    } else {
        if !leaked.is_empty() {
            out.push_str(&format!("LEAKED: {} excluded files found in pub-repo\n", leaked.len()));
            for f in &leaked {
                out.push_str(&format!("  ! {f}\n"));
            }
        }
        if !out_of_sync.is_empty() {
            out.push_str(&format!("OUT-OF-SYNC: {} files differ from dev-repo\n", out_of_sync.len()));
            for f in &out_of_sync {
                out.push_str(&format!("  ~ {f}\n"));
            }
        }
    }

    Ok(out)
}

fn walk_and_check(
    dir: &Path,
    pub_root: &Path,
    repo_cfg: &RepoConfig,
    dev_root: &Path,
    leaked: &mut Vec<String>,
    out_of_sync: &mut Vec<String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(pub_root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy().to_string().replace('\\', "/");

        // Skip .git
        if rel_str == ".git" || rel_str.starts_with(".git/") {
            continue;
        }

        if path.is_dir() {
            // Check if dir itself is excluded
            if is_excluded(rel, repo_cfg) {
                leaked.push(rel_str);
                continue;
            }
            walk_and_check(&path, pub_root, repo_cfg, dev_root, leaked, out_of_sync);
        } else {
            // Check for leaks (files that should be excluded)
            if is_excluded(rel, repo_cfg) {
                leaked.push(rel_str);
                continue;
            }

            // Check sync status
            let dev_file = dev_root.join(rel);
            if dev_file.exists() {
                let pub_content = std::fs::read(&path).unwrap_or_default();
                let dev_content = std::fs::read(&dev_file).unwrap_or_default();
                if pub_content != dev_content {
                    out_of_sync.push(rel_str);
                }
            }
        }
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rulestools-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_parse_publish_config() {
        let dir = make_temp_dir("config");
        let proj = dir.join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(proj.join("rulestools.toml"), r#"
[project]
kind = "tool"

[publish]
targets = ["github", "archive"]
license = "MIT"
author = "Test"

[publish.github]
repo = "user/repo"
"#).unwrap();

        let cfg = PublishConfig::load(&dir);
        assert_eq!(cfg.targets, vec!["github", "archive"]);
        assert_eq!(cfg.license, "MIT");
        assert_eq!(cfg.author, "Test");
        assert!(cfg.github.is_some());
        assert_eq!(cfg.github.unwrap().repo, "user/repo");
        assert!(cfg.forgejo.is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_publish_defaults() {
        let dir = make_temp_dir("defaults");
        let proj = dir.join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(proj.join("rulestools.toml"), "[project]\nkind = \"tool\"\n").unwrap();

        let cfg = PublishConfig::load(&dir);
        assert!(cfg.targets.is_empty());
        assert_eq!(cfg.license, "EUPL-1.2");
        assert_eq!(cfg.author, "TwistedBrain");
        assert!(cfg.github.is_none());
        assert!(cfg.repo.is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_repo_config() {
        let dir = make_temp_dir("repo-cfg");
        let proj = dir.join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(proj.join("rulestools.toml"), r#"
[publish.repo]
path = "../test-pub"
remote = "git@github.com:user/test.git"

[publish.repo.include]
files = ["Cargo.toml", "LICENSE"]
dirs = ["src"]

[publish.repo.exclude]
dirs = ["custom-private"]
patterns = ["*.tmp"]
"#).unwrap();

        let cfg = PublishConfig::load(&dir);
        let repo = cfg.repo.unwrap();
        assert_eq!(repo.path, "../test-pub");
        assert_eq!(repo.remote, "git@github.com:user/test.git");
        assert_eq!(repo.include_files, vec!["Cargo.toml", "LICENSE"]);
        assert_eq!(repo.include_dirs, vec!["src"]);
        assert!(repo.exclude_dirs.contains(&"custom-private".to_string()));
        assert!(repo.exclude_dirs.contains(&"proj".to_string())); // hardcoded
        assert!(repo.exclude_dirs.contains(&".claude".to_string())); // hardcoded
        assert!(repo.exclude_patterns.contains(&"*.tmp".to_string()));
        assert!(repo.exclude_patterns.contains(&".env*".to_string())); // hardcoded

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_version_cargo() {
        let dir = make_temp_dir("version-cargo");
        fs::write(dir.join("Cargo.toml"), r#"
[package]
name = "test"
version = "1.2.3"
"#).unwrap();

        assert_eq!(read_version_cargo(&dir), Some("1.2.3".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_version_workspace() {
        let dir = make_temp_dir("version-ws");
        fs::write(dir.join("Cargo.toml"), r#"
[workspace.package]
version = "0.5.0"
"#).unwrap();

        assert_eq!(read_version_cargo(&dir), Some("0.5.0".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_version_package_json() {
        let dir = make_temp_dir("version-pkg");
        fs::write(dir.join("package.json"), r#"{"name": "test", "version": "2.0.1"}"#).unwrap();

        assert_eq!(read_version_package_json(&dir), Some("2.0.1".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hardcoded_excludes() {
        let dir = make_temp_dir("hardcoded");
        let proj = dir.join("proj");
        fs::create_dir_all(&proj).unwrap();
        // Empty publish.repo — only hardcoded excludes
        fs::write(proj.join("rulestools.toml"), r#"
[publish.repo]
path = "../pub"
remote = "test"
"#).unwrap();

        let cfg = PublishConfig::load(&dir);
        let repo = cfg.repo.unwrap();

        // Hardcoded dirs
        assert!(is_excluded(Path::new("proj/TODO"), &repo));
        assert!(is_excluded(Path::new(".claude/settings.json"), &repo));
        assert!(is_excluded(Path::new("target/debug/main"), &repo));
        assert!(is_excluded(Path::new("doc/readme.md"), &repo));
        assert!(is_excluded(Path::new(".git/config"), &repo));

        // Hardcoded patterns
        assert!(is_excluded(Path::new(".env"), &repo));
        assert!(is_excluded(Path::new(".env.production"), &repo));
        assert!(is_excluded(Path::new("server.key"), &repo));
        assert!(is_excluded(Path::new("api.secret"), &repo));
        assert!(is_excluded(Path::new("TODO"), &repo));
        assert!(is_excluded(Path::new("ISSUES"), &repo));

        // Allowed
        assert!(!is_excluded(Path::new("src/main.rs"), &repo));
        assert!(!is_excluded(Path::new("Cargo.toml"), &repo));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pattern_matches() {
        assert!(pattern_matches("*.key", "server.key"));
        assert!(pattern_matches("*.key", "a.key"));
        assert!(!pattern_matches("*.key", "key"));
        assert!(pattern_matches(".env*", ".env"));
        assert!(pattern_matches(".env*", ".env.production"));
        assert!(!pattern_matches(".env*", "env"));
        assert!(pattern_matches("TODO", "TODO"));
        assert!(!pattern_matches("TODO", "TODOS"));
    }

    #[test]
    fn test_sync_include_only() {
        let dev = make_temp_dir("sync-dev");
        let pub_dir = make_temp_dir("sync-pub");

        // Create dev files
        fs::create_dir_all(dev.join("src")).unwrap();
        fs::write(dev.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dev.join("Cargo.toml"), "[package]\nname=\"t\"").unwrap();
        fs::write(dev.join("secret.key"), "private!").unwrap();

        // Create config
        let proj = dev.join("proj");
        fs::create_dir_all(&proj).unwrap();
        let rel_pub = pub_dir.to_string_lossy().replace('\\', "/");
        fs::write(proj.join("rulestools.toml"), format!(r#"
[publish.repo]
path = "{rel_pub}"
remote = "test"

[publish.repo.include]
files = ["Cargo.toml"]
dirs = ["src"]
"#)).unwrap();

        // Sync preview
        let result = publish_sync(&dev, true).unwrap();
        assert!(result.contains("New:"));

        // Sync for real
        let result = publish_sync(&dev, false).unwrap();
        assert!(result.contains("New:"));

        // Verify: included files exist
        assert!(pub_dir.join("Cargo.toml").exists());
        assert!(pub_dir.join("src/main.rs").exists());

        // Verify: excluded files DO NOT exist
        assert!(!pub_dir.join("secret.key").exists());
        assert!(!pub_dir.join("proj").exists());

        let _ = fs::remove_dir_all(&dev);
        let _ = fs::remove_dir_all(&pub_dir);
    }

    #[test]
    fn test_sync_exclude_blocks() {
        let dev = make_temp_dir("sync-excl");
        let pub_dir = make_temp_dir("sync-excl-pub");

        // Create dev files including excluded ones
        fs::create_dir_all(dev.join("src")).unwrap();
        fs::write(dev.join("src/main.rs"), "fn main() {}").unwrap();
        fs::create_dir_all(dev.join("src/.claude")).unwrap();
        fs::write(dev.join("src/.env"), "SECRET=x").unwrap();

        let proj = dev.join("proj");
        fs::create_dir_all(&proj).unwrap();
        let rel_pub = pub_dir.to_string_lossy().replace('\\', "/");
        fs::write(proj.join("rulestools.toml"), format!(r#"
[publish.repo]
path = "{rel_pub}"
remote = "test"

[publish.repo.include]
dirs = ["src"]
"#)).unwrap();

        publish_sync(&dev, false).unwrap();

        assert!(pub_dir.join("src/main.rs").exists());
        assert!(!pub_dir.join("src/.env").exists()); // pattern exclude
        assert!(!pub_dir.join("src/.claude").exists()); // dir exclude

        let _ = fs::remove_dir_all(&dev);
        let _ = fs::remove_dir_all(&pub_dir);
    }

    #[test]
    fn test_check_detects_leak() {
        let dev = make_temp_dir("check-leak");
        let pub_dir = make_temp_dir("check-leak-pub");

        let proj = dev.join("proj");
        fs::create_dir_all(&proj).unwrap();
        let rel_pub = pub_dir.to_string_lossy().replace('\\', "/");
        fs::write(proj.join("rulestools.toml"), format!(r#"
[publish.repo]
path = "{rel_pub}"
remote = "test"
"#)).unwrap();

        // Manually place a leaked file in pub-repo
        fs::write(pub_dir.join(".env"), "LEAKED=true").unwrap();
        fs::create_dir_all(pub_dir.join("proj")).unwrap();
        fs::write(pub_dir.join("proj/TODO"), "leaked task").unwrap();

        let result = publish_check(&dev).unwrap();
        assert!(result.contains("LEAKED"));
        assert!(result.contains(".env"));

        let _ = fs::remove_dir_all(&dev);
        let _ = fs::remove_dir_all(&pub_dir);
    }

    #[test]
    fn test_check_clean() {
        let dev = make_temp_dir("check-clean");
        let pub_dir = make_temp_dir("check-clean-pub");

        let proj = dev.join("proj");
        fs::create_dir_all(&proj).unwrap();
        let rel_pub = pub_dir.to_string_lossy().replace('\\', "/");
        fs::write(proj.join("rulestools.toml"), format!(r#"
[publish.repo]
path = "{rel_pub}"
remote = "test"
"#)).unwrap();

        // Put only allowed files in pub-repo
        fs::write(pub_dir.join("README.md"), "# Test").unwrap();

        // Create matching file in dev
        fs::write(dev.join("README.md"), "# Test").unwrap();

        let result = publish_check(&dev).unwrap();
        assert!(result.contains("CLEAN"));

        let _ = fs::remove_dir_all(&dev);
        let _ = fs::remove_dir_all(&pub_dir);
    }
}
