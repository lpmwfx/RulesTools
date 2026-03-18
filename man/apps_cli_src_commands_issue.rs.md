# apps/cli/src/commands/issue.rs

## `pub fn forgejo_token() -> Result<String, String>`

*Line 4 · fn*

fn `forgejo_token`.

---

## `pub fn resolve_label_ids(token: &str, names: &[&str]) -> Result<Vec<u64>, String>`

*Line 10 · fn*

fn `resolve_label_ids`.

---

## `pub fn report_internal(title: &str, body: &str, labels_str: &str) -> Result<String, String>`

*Line 35 · fn*

fn `report_internal`.

---

## `pub fn list_internal(state: &str, labels_str: &str) -> Result<String, String>`

*Line 63 · fn*

fn `list_internal`.

---

## `pub fn close_internal(number: u64, comment: &str) -> Result<String, String>`

*Line 98 · fn*

fn `close_internal`.

---

## `pub fn add_label_internal(number: u64, label: &str) -> Result<String, String>`

*Line 120 · fn*

fn `add_label_internal`.

---

## `pub fn create_label_internal(name: &str, color: &str, description: &str) -> Result<String, String>`

*Line 139 · fn*

fn `create_label_internal`.

---

## `pub fn list_labels_internal() -> Result<String, String>`

*Line 163 · fn*

fn `list_labels_internal`.

---

## `pub fn comment_internal(number: u64, body: &str) -> Result<String, String>`

*Line 193 · fn*

fn `comment_internal`.

---

## `pub fn cmd_issue_report(title: &str, body: &str, labels: &str)`

*Line 209 · fn*

fn `cmd_issue_report`.

---

## `pub fn cmd_issue_list(state: &str, labels: &str)`

*Line 217 · fn*

fn `cmd_issue_list`.

---

## `pub fn cmd_issue_close(number: u64, comment: &str)`

*Line 225 · fn*

fn `cmd_issue_close`.

---

## `pub fn cmd_issue_add_label(number: u64, label: &str)`

*Line 233 · fn*

fn `cmd_issue_add_label`.

---

## `pub fn cmd_issue_create_label(name: &str, color: &str, description: &str)`

*Line 241 · fn*

fn `cmd_issue_create_label`.

---

## `pub fn cmd_issue_list_labels()`

*Line 249 · fn*

fn `cmd_issue_list_labels`.

---

## `pub fn cmd_issue_comment(number: u64, body: &str)`

*Line 257 · fn*

fn `cmd_issue_comment`.

---

## `pub fn cmd_issue(cmd: IssueCmd)`

*Line 268 · fn*

fn `cmd_issue`.

---

