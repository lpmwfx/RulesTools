use std::path::Path;

use rulestools_scanner::project::ProjectKind;

/// Target platform for SlintApp/Super projects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Mobile,
    Small,
}

impl Platform {
    /// fn `from_str`.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "mobile" => Some(Self::Mobile),
            "small" => Some(Self::Small),
            _ => None,
        }
    }

    /// fn `name`.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
            Self::Small => "small",
        }
    }
}

/// Extra folders/crates to scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extra {
    Lib,
    Shared,
    Doc,
}

impl Extra {
    /// fn `from_str`.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "lib" => Some(Self::Lib),
            "shared" => Some(Self::Shared),
            "doc" => Some(Self::Doc),
            _ => None,
        }
    }
}

/// Options for `rulestools new`.
pub struct ScaffoldOptions {
    pub name: String,
    pub kind: ProjectKind,
    pub platforms: Vec<Platform>,
    pub themes: Vec<String>,
    pub mcp: bool,
    pub extras: Vec<Extra>,
    pub preview: bool,
}

/// Result of a scaffold/update operation.
pub struct ScaffoldResult {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
    pub summary: String,
}

/// Options for `rulestools update`.
pub struct UpdateOptions {
    pub platforms: Vec<Platform>,
    pub themes: Vec<String>,
    pub crate_name: Option<String>,
    pub folders: Vec<Extra>,
    pub preview: bool,
}

/// Move guidance for project upgrades.
#[derive(Debug)]
pub struct MoveGuidance {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// Result of a project upgrade.
#[derive(Debug)]
pub struct UpgradeResult {
    pub from_kind: ProjectKind,
    pub to_kind: ProjectKind,
    pub created: Vec<String>,
    pub move_guidance: Vec<MoveGuidance>,
    pub manual_steps: Vec<String>,
}

// --- Writer (dry_run support) ---

pub(super) struct Writer {
    pub(super) dry_run: bool,
}

impl Writer {
    pub(super) fn ensure_dir(&self, dir: &Path, created: &mut Vec<String>) -> Result<(), String> {
        if !dir.exists() {
            if !self.dry_run {
                std::fs::create_dir_all(dir)
                    .map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
            }
            created.push(format!("{}/", dir.display()));
        }
        Ok(())
    }

    pub(super) fn write_if_missing(
        &self,
        dir: &Path,
        filename: &str,
        content: &str,
        created: &mut Vec<String>,
    ) -> Result<(), String> {
        let path = dir.join(filename);
        if !path.exists() {
            if !self.dry_run {
                std::fs::write(&path, content)
                    .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
            }
            created.push(format!("{}", path.display()));
        }
        Ok(())
    }

}
