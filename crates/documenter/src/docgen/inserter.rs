use std::path::Path;

use crate::manifest::SourceDoc;
use super::DocGenResult;

/// Process all source docs: find undocumented items, call AI, insert docs.
pub fn process_all(root: &Path, docs: &[SourceDoc]) -> DocGenResult {
    let mut result = DocGenResult {
        generated: 0,
        skipped: 0,
        log: Vec::new(),
    };

    for doc in docs {
        let undoc_count = doc.items.iter().filter(|i| !i.is_documented()).count();
        if undoc_count == 0 {
            continue;
        }

        let file_path = root.join(&doc.source);
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                result.log.push(format!("skip {}: {e}", doc.source));
                result.skipped += undoc_count;
                continue;
            }
        };

        let (rewritten, count) = rewrite_with_docs(&content, doc, &mut result);
        if count > 0 {
            let _ = std::fs::write(&file_path, &rewritten);
        }
    }

    result
}

/// Rewrite file content by inserting AI-generated doc comments.
/// Processes items in reverse line order to preserve line numbers.
fn rewrite_with_docs(content: &str, doc: &SourceDoc, result: &mut DocGenResult) -> (String, usize) {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut count = 0usize;

    // Undocumented items, sorted by line descending
    let mut undoc: Vec<_> = doc.items.iter().filter(|i| !i.is_documented()).collect();
    undoc.sort_by(|a, b| b.line.cmp(&a.line));

    for item in &undoc {
        let doc_text = generate_stub_doc(item.kind.label(), &item.name);
        insert_doc_lines(&mut lines, item.line, &doc_text);
        result.generated += 1;
        result.log.push(format!("documented {} `{}` in {}", item.kind.label(), item.name, doc.source));
        count += 1;
    }

    // Preserve trailing newline
    let mut output = lines.join("\n");
    if content.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }

    (output, count)
}

/// Generate a stub doc comment (without AI — uses deterministic template).
fn generate_stub_doc(kind: &str, name: &str) -> String {
    format!("{kind} `{name}`.")
}

/// Insert `///` doc comment lines above the item at `item_line` (1-based).
fn insert_doc_lines(lines: &mut Vec<String>, item_line: usize, doc_text: &str) {
    if item_line == 0 || item_line > lines.len() {
        return;
    }
    let idx = item_line - 1;
    let indent = lines[idx]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    let doc_lines: Vec<String> = doc_text
        .lines()
        .map(|l| {
            if l.is_empty() {
                format!("{indent}///")
            } else {
                format!("{indent}/// {l}")
            }
        })
        .collect();

    for (offset, doc_line) in doc_lines.into_iter().enumerate() {
        lines.insert(idx + offset, doc_line);
    }
}
