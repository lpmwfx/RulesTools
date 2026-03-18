# apps/cli/src/publish.rs

## `pub struct PublishConfig`

*Line 6 · struct*

struct `PublishConfig`.

---

## `pub struct GithubTarget`

*Line 17 · struct*

struct `GithubTarget`.

---

## `pub struct ForgejoTarget`

*Line 22 · struct*

struct `ForgejoTarget`.

---

## `pub struct RepoConfig`

*Line 28 · struct*

struct `RepoConfig`.

---

## `pub fn load(root: &Path) -> Self`

*Line 49 · fn*

fn `load`.

---

## `pub struct PublishPlan`

*Line 353 · struct*

struct `PublishPlan`.

---

## `pub struct TargetPlan`

*Line 365 · struct*

struct `TargetPlan`.

---

## `pub fn publish_plan(root: &Path, format: &str) -> Result<String, String>`

*Line 371 · fn*

fn `publish_plan`.

---

## `pub fn publish_status(root: &Path, format: &str) -> Result<String, String>`

*Line 490 · fn*

fn `publish_status`.

---

## `pub fn publish_run(root: &Path, target: &str, preview: bool) -> Result<String, String>`

*Line 609 · fn*

fn `publish_run`.

---

## `pub fn publish_init(root: &Path, remote: &str) -> Result<String, String>`

*Line 851 · fn*

fn `publish_init`.

---

## `pub fn publish_sync(root: &Path, preview: bool) -> Result<String, String>`

*Line 904 · fn*

fn `publish_sync`.

---

## `pub fn publish_check(root: &Path) -> Result<String, String>`

*Line 1064 · fn*

fn `publish_check`.

---

