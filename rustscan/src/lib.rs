//! # rustscan
//!
//! Unified wrapper for RustScanners.
//! Delegates all calls to the upstream `rustscanners` crate.
//!
//! Add to `Cargo.toml` `[build-dependencies]`:
//! ```toml
//! rustscan = { git = "https://github.com/lpmwfx/RulesTools" }
//! ```
//!
//! Call from `build.rs`:
//! ```rust,ignore
//! fn main() {
//!     rustscan::scan_project();
//! }
//! ```

/// Scan the Rust project and emit `cargo:warning` for each violation.
///
/// Returns the total number of errors found.
/// Call this from `build.rs`.
pub fn scan_project() -> usize {
    rustscanners::scan_project()
}
