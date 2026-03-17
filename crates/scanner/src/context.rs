use std::path::Path;

/// Supported source languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Slint,
    Python,
    JavaScript,
    TypeScript,
    Css,
    Kotlin,
    CSharp,
    Html,
}

impl Language {
    /// Detect language from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "slint" => Some(Language::Slint),
            "py" => Some(Language::Python),
            "js" | "mjs" | "cjs" | "jsx" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "css" | "scss" => Some(Language::Css),
            "kt" | "kts" => Some(Language::Kotlin),
            "cs" => Some(Language::CSharp),
            "html" | "htm" => Some(Language::Html),
            _ => None,
        }
    }

    /// Detect language from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    pub fn name(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Slint => "slint",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Css => "css",
            Language::Kotlin => "kotlin",
            Language::CSharp => "csharp",
            Language::Html => "html",
        }
    }
}

/// Context for a single file being scanned.
#[derive(Debug, Clone)]
pub struct FileContext {
    pub language: Language,
    pub is_test_file: bool,
    pub is_mother_file: bool,
    pub is_definition_file: bool,
}

impl FileContext {
    /// Build context from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let lang = Language::from_path(path)?;
        let filename = path.file_name()?.to_str()?;
        let path_str = path.to_string_lossy();

        let is_test_file = match lang {
            Language::Rust => {
                filename.starts_with("test_")
                    || filename.ends_with("_test.rs")
                    || filename.ends_with("_tests.rs")
                    || path_str.contains("/tests/")
                    || path_str.contains("\\tests\\")
                    || filename == "tests.rs"
            }
            Language::Python => {
                filename.starts_with("test_")
                    || filename.ends_with("_test.py")
                    || path_str.contains("/tests/")
                    || path_str.contains("\\tests\\")
            }
            Language::JavaScript | Language::TypeScript => {
                filename.contains(".test.")
                    || filename.contains(".spec.")
                    || path_str.contains("/__tests__/")
                    || path_str.contains("\\__tests__\\")
            }
            _ => false,
        };

        let is_mother_file = match lang {
            Language::Rust => filename == "mod.rs" || filename == "main.rs" || filename == "lib.rs",
            Language::Slint => {
                // Window components or files that aggregate children
                filename.ends_with("_view.slint") || filename == "main.slint"
            }
            _ => false,
        };

        let is_definition_file = match lang {
            Language::Slint => {
                filename.starts_with("_") || path_str.contains("/globals/") || path_str.contains("\\globals\\")
            }
            _ => false,
        };

        Some(Self {
            language: lang,
            is_test_file,
            is_mother_file,
            is_definition_file,
        })
    }
}

/// Check if a line is a comment in the given language.
pub fn is_comment(line: &str, lang: Language) -> bool {
    let trimmed = line.trim();
    match lang {
        Language::Rust | Language::Slint | Language::JavaScript | Language::TypeScript
        | Language::Kotlin | Language::CSharp | Language::Css => {
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
        }
        Language::Html => trimmed.starts_with("<!--"),
        Language::Python => trimmed.starts_with('#'),
    }
}

/// Check if a line is a const/static definition (Rust).
pub fn is_const_def(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("const ") || trimmed.starts_with("static ") || trimmed.starts_with("pub const ") || trimmed.starts_with("pub static ")
}

/// Check if lines around an index indicate a test context (Rust #[test] or #[cfg(test)]).
pub fn is_test_context(lines: &[&str], index: usize) -> bool {
    let start = index.saturating_sub(60);
    for i in (start..index).rev() {
        let trimmed = lines[i].trim();
        if trimmed == "#[test]" || trimmed == "#[cfg(test)]" || trimmed.starts_with("#[rstest") {
            return true;
        }
        // Stop at fn boundary — but peek one line up for #[test] first
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") {
            if i > 0 {
                let above = lines[i - 1].trim();
                if above == "#[test]" || above == "#[cfg(test)]" || above.starts_with("#[rstest") {
                    return true;
                }
            }
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("slint"), Some(Language::Slint));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("jsx"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("css"), Some(Language::Css));
        assert_eq!(Language::from_extension("scss"), Some(Language::Css));
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));
        assert_eq!(Language::from_extension("html"), Some(Language::Html));
        assert_eq!(Language::from_extension("htm"), Some(Language::Html));
        assert_eq!(Language::from_extension("txt"), None);
        assert_eq!(Language::from_extension("md"), None);
    }

    #[test]
    fn language_from_path() {
        assert_eq!(Language::from_path(&PathBuf::from("src/main.rs")), Some(Language::Rust));
        assert_eq!(Language::from_path(&PathBuf::from("app.slint")), Some(Language::Slint));
        assert_eq!(Language::from_path(&PathBuf::from("README.md")), None);
    }

    #[test]
    fn rust_test_file_detection() {
        let ctx = FileContext::from_path(&PathBuf::from("tests/test_main.rs")).unwrap();
        assert!(ctx.is_test_file);

        let ctx = FileContext::from_path(&PathBuf::from("src/main.rs")).unwrap();
        assert!(!ctx.is_test_file);

        let ctx = FileContext::from_path(&PathBuf::from("src/foo_test.rs")).unwrap();
        assert!(ctx.is_test_file);
    }

    #[test]
    fn slint_mother_detection() {
        let ctx = FileContext::from_path(&PathBuf::from("ui/main_view.slint")).unwrap();
        assert!(ctx.is_mother_file);

        let ctx = FileContext::from_path(&PathBuf::from("ui/main.slint")).unwrap();
        assert!(ctx.is_mother_file);

        let ctx = FileContext::from_path(&PathBuf::from("ui/button.slint")).unwrap();
        assert!(!ctx.is_mother_file);
    }

    #[test]
    fn slint_definition_detection() {
        let ctx = FileContext::from_path(&PathBuf::from("ui/_tokens.slint")).unwrap();
        assert!(ctx.is_definition_file);

        let ctx = FileContext::from_path(&PathBuf::from("ui/globals/theme.slint")).unwrap();
        assert!(ctx.is_definition_file);
    }

    #[test]
    fn rust_mother_file() {
        let ctx = FileContext::from_path(&PathBuf::from("src/mod.rs")).unwrap();
        assert!(ctx.is_mother_file);

        let ctx = FileContext::from_path(&PathBuf::from("src/lib.rs")).unwrap();
        assert!(ctx.is_mother_file);
    }

    #[test]
    fn is_const_def_check() {
        assert!(is_const_def("const FOO: i32 = 42;"));
        assert!(is_const_def("pub const BAR: &str = \"x\";"));
        assert!(is_const_def("static COUNTER: AtomicUsize = AtomicUsize::new(0);"));
        assert!(is_const_def("pub static REF: &str = \"y\";"));
        assert!(!is_const_def("let x = 5;"));
        assert!(!is_const_def("fn constant() {}"));
    }

    #[test]
    fn is_comment_check() {
        assert!(is_comment("  // hello", Language::Rust));
        assert!(is_comment("  /* block */", Language::Rust));
        assert!(is_comment("  * continuation", Language::Rust));
        assert!(is_comment("# comment", Language::Python));
        assert!(!is_comment("let x = 5;", Language::Rust));
        assert!(!is_comment("x = 5", Language::Python));
    }

    #[test]
    fn test_context_detection() {
        let lines = vec![
            "",
            "    #[test]",
            "    fn test_foo() {",
            "        let x = 42;",
            "    }",
        ];
        // Line 2 (fn test_foo) sees #[test] on line 1
        assert!(is_test_context(&lines, 2));
        // Line 3 (let x) is inside test fn — but hits fn boundary, so stops
        // The fn boundary IS the test fn, so #[test] was already found
        assert!(is_test_context(&lines, 3));
        // Line 0 (empty) has nothing above
        assert!(!is_test_context(&lines, 0));
    }
}
