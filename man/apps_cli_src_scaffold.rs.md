# apps/cli/src/scaffold.rs

## `pub enum Platform`

*Line 35 · enum*

Target platform for SlintApp/Super projects.

---

## `pub fn from_str(s: &str) -> Option<Self>`

*Line 43 · fn*

fn `from_str`.

---

## `pub fn name(&self) -> &'static str`

*Line 53 · fn*

fn `name`.

---

## `pub enum Extra`

*Line 64 · enum*

Extra folders/crates to scaffold.

---

## `pub fn from_str(s: &str) -> Option<Self>`

*Line 72 · fn*

fn `from_str`.

---

## `pub struct ScaffoldOptions`

*Line 83 · struct*

Options for `rulestools new`.

---

## `pub struct ScaffoldResult`

*Line 94 · struct*

Result of a scaffold/update operation.

---

## `pub struct UpdateOptions`

*Line 101 · struct*

Options for `rulestools update`.

---

## `pub struct MoveGuidance`

*Line 111 · struct*

Move guidance for project upgrades.

---

## `pub struct UpgradeResult`

*Line 119 · struct*

Result of a project upgrade.

---

## `pub fn scaffold_project(root: &Path, kind: ProjectKind, name: &str) -> Result<String, String>`

*Line 168 · fn*

Scaffold a full project structure for the given kind (backward compat).

---

## `pub fn scaffold_with_options(root: &Path, opts: &ScaffoldOptions) -> Result<ScaffoldResult, String>`

*Line 189 · fn*

Scaffold a project with full options (new_project).

---

## `pub fn update_project(root: &Path, opts: &UpdateOptions) -> Result<ScaffoldResult, String>`

*Line 249 · fn*

Update an existing project — add features within current kind.

Also checks for missing integration components (hooks, build.rs, topology)
and adds them if absent.

---

## `pub fn upgrade_project( root: &Path, to_kind: ProjectKind, preview: bool, ) -> Result<UpgradeResult, String>`

*Line 382 · fn*

Upgrade a project to a higher kind.

---

## `pub fn render_tree(root_name: &str, paths: &[String]) -> String`

*Line 479 · fn*

Render a simple directory tree from a list of created paths.

---

## `pub fn init() {{\n\`

*Line 785 · fn*

Initialize {} platform.\n\

---

## `pub fn init() {{\n\`

*Line 802 · fn*

Initialize {} platform.\n\

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
