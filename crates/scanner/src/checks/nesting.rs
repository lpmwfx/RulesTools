use std::path::Path;

use crate::config::Config;
use crate::context::{FileContext, Language};
use crate::issue::{Issue, Severity};

/// Max nesting depth per language.
fn max_depth(lang: Language) -> usize {
    match lang {
        Language::Rust => 5,
        Language::JavaScript | Language::TypeScript | Language::Css => 4,
        Language::Html => 4,
        Language::Slint | Language::Kotlin => 6,
        Language::CSharp => 7,
        Language::Python => 8,
    }
}

/// Check control-flow nesting depth — measures complexity, not just braces.
///
/// Counts nesting for control flow (`if`, `for`, `while`, `match`, `loop`,
/// closures, callbacks) but NOT for struct/enum bodies, type annotations,
/// or Slint property types.
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let limit = max_depth(file_ctx.language);

    // Python uses indentation-based nesting
    if file_ctx.language == Language::Python {
        check_python_nesting(lines, limit, issues, path);
        return;
    }

    // Brace-based languages: track total brace depth AND flow depth separately.
    // total_depth tracks ALL braces (to know when a `}` closes a flow vs non-flow brace).
    // flow_depth tracks only control-flow nesting (what we report on).
    let mut total_depth: usize = 0;
    let mut flow_depth: usize = 0;
    // Stack: true = this brace level is control-flow, false = structural
    let mut brace_stack: Vec<bool> = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }

        let flow_opens = count_flow_opens(trimmed, file_ctx.language);
        let total_opens = count_all_opens(trimmed, file_ctx.language);
        let closes = count_closes(trimmed, file_ctx.language);

        // Push each opening brace onto stack
        for i in 0..total_opens {
            let is_flow = i < flow_opens;
            brace_stack.push(is_flow);
            total_depth += 1;
            if is_flow {
                flow_depth += 1;
            }
        }

        if flow_depth > limit {
            let sev = if flow_depth > limit + 1 {
                Severity::Error
            } else {
                Severity::Warning
            };
            issues.push(Issue::new(
                path,
                line_num + 1,
                1,
                sev,
                "global/nesting",
                format!("nesting depth {flow_depth} exceeds limit {limit} — extract a helper function"),
            ));
        }

        // Pop closing braces from stack
        for _ in 0..closes {
            if let Some(was_flow) = brace_stack.pop() {
                total_depth = total_depth.saturating_sub(1);
                if was_flow {
                    flow_depth = flow_depth.saturating_sub(1);
                }
            }
        }
    }
}

/// Count ALL opening braces on a line (for brace-stack tracking).
fn count_all_opens(line: &str, lang: Language) -> usize {
    let trimmed = line.trim();
    if lang == Language::Slint && trimmed.contains("property") && trimmed.contains('<') {
        return 0; // Slint property type annotations
    }
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        return 0;
    }
    trimmed.matches('{').count()
}

/// Count control-flow opening constructs on a line.
/// Only counts things that add complexity — not struct/type/annotation braces.
fn count_flow_opens(line: &str, lang: Language) -> usize {
    let trimmed = line.trim();

    match lang {
        Language::Rust => count_rust_flow_opens(trimmed),
        Language::Slint => count_slint_flow_opens(trimmed),
        Language::JavaScript | Language::TypeScript => count_js_flow_opens(trimmed),
        Language::Python => count_python_flow_opens(trimmed),
        _ => count_generic_flow_opens(trimmed),
    }
}

/// Rust: count if/else/match/for/while/loop/fn/closure openers.
fn count_rust_flow_opens(line: &str) -> usize {
    // Skip struct/enum/impl/trait/type definitions — not control flow
    if line.starts_with("pub struct ")
        || line.starts_with("struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("enum ")
        || line.starts_with("impl ")
        || line.starts_with("pub trait ")
        || line.starts_with("trait ")
        || line.starts_with("type ")
        || line.starts_with("pub type ")
        || line.starts_with("const ")
        || line.starts_with("pub const ")
        || line.starts_with("static ")
        || line.starts_with("pub static ")
        || line.starts_with("mod ")
        || line.starts_with("pub mod ")
        || line.starts_with("#[")
    {
        return 0;
    }

    let mut count = 0;

    // Function/method definition — counts as one nesting level
    if (line.starts_with("fn ") || line.starts_with("pub fn ")
        || line.starts_with("async fn ") || line.starts_with("pub async fn ")
        || line.starts_with("pub(crate) fn ") || line.starts_with("pub(super) fn "))
        && line.contains('{')
    {
        count += 1;
    }

    // Control flow keywords that open a block
    if line.contains('{') {
        for keyword in ["if ", "else {", "else if ", "match ", "for ", "while ", "loop {", "loop{"] {
            if line.contains(keyword) {
                count += 1;
                break; // only count one per line (if + else on same line = 1)
            }
        }

        // Closures: |...| { or move |...| {
        if line.contains("| {") || line.contains("|{") {
            count += 1;
        }
    }

    count
}

/// Slint: count component/if/for/callback openers. Skip property types.
fn count_slint_flow_opens(line: &str) -> usize {
    // Skip property declarations with type annotations
    if line.contains("property") && line.contains('<') {
        return 0;
    }
    // Skip import/export statements
    if line.starts_with("import ") || line.starts_with("export ") && !line.contains('{') {
        return 0;
    }

    let mut count = 0;

    if line.contains('{') {
        // Component definition: `component Foo inherits Bar {`
        if line.contains("component ") || line.contains("inherits ") {
            count += 1;
        }
        // Control flow
        else if line.contains("if ") || line.contains("for ") {
            count += 1;
        }
        // Callback body: `=> {`
        else if line.contains("=> {") || line.contains("=>{") {
            count += 1;
        }
        // Animation/transition/states
        else if line.contains("animate ") || line.contains("states [") {
            count += 1;
        }
        // Nested element (Rectangle {, Text {, etc.) — these are layout nesting
        else if line.ends_with('{') || line.ends_with("{ ") {
            // Only count if it looks like an element (starts with uppercase or known element)
            let first_word = line.split_whitespace().next().unwrap_or("");
            if first_word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                || first_word == "root"
            {
                count += 1;
            }
        }
    }

    count
}

/// JavaScript/TypeScript: count if/else/for/while/switch/function/arrow openers.
fn count_js_flow_opens(line: &str) -> usize {
    if line.contains('{') {
        for keyword in ["if ", "else {", "else if ", "for ", "while ", "switch ", "function ", "=> {", "=>{", "catch "] {
            if line.contains(keyword) {
                return 1;
            }
        }
    }
    0
}

/// Python nesting — measured by indentation level on control-flow lines.
fn check_python_nesting(lines: &[&str], limit: usize, issues: &mut Vec<Issue>, path: &Path) {
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Only check control-flow lines
        let is_flow = ["if ", "elif ", "else:", "for ", "while ", "with ", "try:",
            "except ", "finally:", "def ", "class ", "async def ", "async for ", "async with "]
            .iter()
            .any(|kw| trimmed.starts_with(kw));

        if !is_flow {
            continue;
        }

        // Calculate indentation depth (spaces / 4, tabs count as 1)
        let indent: usize = line.chars().take_while(|c| c.is_whitespace()).map(|c| {
            if c == '\t' { 4 } else { 1 }
        }).sum::<usize>() / 4;

        if indent > limit {
            let sev = if indent > limit + 1 { Severity::Error } else { Severity::Warning };
            issues.push(Issue::new(
                path,
                line_num + 1,
                1,
                sev,
                "global/nesting",
                format!("nesting depth {indent} exceeds limit {limit} — extract a helper function"),
            ));
        }
    }
}

/// Python: count if/for/while/with/try/def/class — uses indentation, not braces.
fn count_python_flow_opens(line: &str) -> usize {
    for keyword in ["if ", "elif ", "else:", "for ", "while ", "with ", "try:", "except ", "def ", "class ", "async def ", "async for ", "async with "] {
        if line.starts_with(keyword) {
            return 1;
        }
    }
    0
}

/// Generic: count lines that open braces with control-flow keywords.
fn count_generic_flow_opens(line: &str) -> usize {
    if line.contains('{') {
        for keyword in ["if ", "else", "for ", "while ", "switch ", "case "] {
            if line.contains(keyword) {
                return 1;
            }
        }
    }
    0
}

/// Count depth reductions — language-aware.
fn count_closes(line: &str, lang: Language) -> usize {
    let trimmed = line.trim();

    match lang {
        Language::Python => {
            // Python uses indentation, not braces — dedent tracked differently.
            // We track opens per keyword, closes must match.
            // A line that starts a new block at same/lower indent closes the previous.
            // For simplicity: Python uses open-only tracking — each keyword increments,
            // and we rely on the function boundary to reset.
            // Don't count } at all for Python.
            0
        }
        Language::Slint => {
            // Skip property type annotations
            if trimmed.contains("property") && trimmed.contains('<') {
                return 0;
            }
            trimmed.matches('}').count()
        }
        _ => {
            // Skip string literals
            if trimmed.starts_with('"') || trimmed.starts_with('\'') {
                return 0;
            }
            trimmed.matches('}').count()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_ctx(lang: Language) -> FileContext {
        FileContext {
            language: lang,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn normal_rust_nesting_ok() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn main() {",
            "    if true {",
            "        println!(\"ok\");",
            "    }",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn deep_rust_nesting_flagged() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn f() {",
            "  if a {",
            "    if b {",
            "      if c {",
            "        if d {",
            "          if e {",
            "          }",
            "        }",
            "      }",
            "    }",
            "  }",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(!issues.is_empty());
    }

    #[test]
    fn struct_def_not_counted() {
        let mut issues = Vec::new();
        let lines = vec![
            "pub struct Foo {",
            "    field: i32,",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn enum_def_not_counted() {
        let mut issues = Vec::new();
        let lines = vec![
            "pub enum Kind {",
            "    A,",
            "    B { inner: i32 },",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn impl_block_not_counted() {
        let mut issues = Vec::new();
        let lines = vec![
            "impl Foo {",
            "    fn bar() {",
            "    }",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn slint_property_type_not_counted() {
        let mut issues = Vec::new();
        let lines = vec![
            "export component Foo {",
            "    in property <[{color: brush, label: string}]> items;",
            "    in property <{x: int, y: int}> pos;",
            "}",
        ];
        check(&make_ctx(Language::Slint), &lines, &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(issues.is_empty(), "got {} issues: {:?}", issues.len(), issues.iter().map(|i| &i.message).collect::<Vec<_>>());
    }

    #[test]
    fn slint_nested_components_counted() {
        let mut issues = Vec::new();
        let lines = vec![
            "export component App inherits Window {",
            "  VerticalLayout {",
            "    HorizontalLayout {",
            "      Rectangle {",
            "        VerticalLayout {",
            "          Text {",
            "            Rectangle {",
            "              Text { }",
            "            }",
            "          }",
            "        }",
            "      }",
            "    }",
            "  }",
            "}",
        ];
        check(&make_ctx(Language::Slint), &lines, &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(!issues.is_empty()); // 7 levels deep, limit is 6
    }

    #[test]
    fn rust_match_counts_as_nesting() {
        assert_eq!(count_rust_flow_opens("    match self {"), 1);
    }

    #[test]
    fn rust_struct_not_nesting() {
        assert_eq!(count_rust_flow_opens("pub struct Config {"), 0);
        assert_eq!(count_rust_flow_opens("struct Inner {"), 0);
    }

    #[test]
    fn rust_fn_counts() {
        assert_eq!(count_rust_flow_opens("fn main() {"), 1);
        assert_eq!(count_rust_flow_opens("pub fn check() {"), 1);
    }
}
