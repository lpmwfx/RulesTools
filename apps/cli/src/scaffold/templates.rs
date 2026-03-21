pub(super) const SCANNER_BUILD_DEP: &str = "rulestools-scanner = { git = \"https://github.com/lpmwfx/RulesTools\" }";
pub(super) const BUILD_RS_SCANNER: &str = "fn main() {\n    rulestools_scanner::scan_project();\n}\n";
pub(super) const BUILD_RS_SCANNER_SLINT: &str = "fn main() {\n    rulestools_scanner::scan_project();\n    slint_build::compile(\"ui/main.slint\").expect(\"Slint build failed\");\n}\n";

/// CLI topology folders (no UI).
pub(super) const CLI_TOPOLOGY: &[&str] = &["core", "adapter", "gateway", "pal", "shared"];

/// GUI topology folders (CLI + app + ui).
pub(super) const GUI_TOPOLOGY: &[&str] = &["app", "core", "adapter", "gateway", "pal", "shared", "ui"];

pub(super) const CLAUDE_SETTINGS: &str = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "rulestools hook"
          }
        ]
      }
    ]
  }
}
"#;

pub(super) const PRE_COMMIT_HOOK: &str = "#!/bin/sh\nrulestools check \"$(git rev-parse --show-toplevel)\"\n";

pub(super) fn slint_main_content(name: &str) -> String {
    format!(
        "import {{ Button, VerticalBox }} from \"std-widgets.slint\";\n\n\
         export component App inherits Window {{\n\
             title: \"{name}\";\n\
             width: 800px;\n\
             height: 600px;\n\n\
             VerticalBox {{\n\
                 Button {{ text: \"Hello\"; }}\n\
             }}\n\
         }}\n"
    )
}

pub(super) fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c == ' ')
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

pub(super) fn gitignore_content() -> &'static str {
    "/target\n*.swp\n*.swo\n*~\n.DS_Store\nThumbs.db\n"
}

pub(super) fn cargo_toml_bin(name: &str, extra_deps: &[&str], build_deps: &[&str]) -> String {
    let mut s = format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\n\
         [dependencies]\n"
    );
    for dep in extra_deps {
        s.push_str(dep);
        s.push('\n');
    }
    if !build_deps.is_empty() {
        s.push_str("\n[build-dependencies]\n");
        for dep in build_deps {
            s.push_str(dep);
            s.push('\n');
        }
    }
    s
}

pub(super) fn cargo_toml_lib(name: &str, extra_deps: &[&str], build_deps: &[&str]) -> String {
    let mut s = format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\n\
         [lib]\n\
         name = \"{}\"\n\
         path = \"src/lib.rs\"\n\n\
         [dependencies]\n",
        name.replace('-', "_"),
    );
    for dep in extra_deps {
        s.push_str(dep);
        s.push('\n');
    }
    if !build_deps.is_empty() {
        s.push_str("\n[build-dependencies]\n");
        for dep in build_deps {
            s.push_str(dep);
            s.push('\n');
        }
    }
    s
}
