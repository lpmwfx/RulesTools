//! # slintscan
//!
//! Unified wrapper for SlintScanners.
//! Delegates all calls to the upstream `slintscanners` crate.
//!
//! Add to `Cargo.toml` `[build-dependencies]`:
//! ```toml
//! slintscan = { git = "https://github.com/lpmwfx/RulesTools" }
//! ```
//!
//! Call from `build.rs`:
//! ```rust,ignore
//! fn main() {
//!     slintscan::scan_project();
//! }
//! ```

/// Scan all `.slint` files and emit `cargo:warning` for each violation.
///
/// Returns the total number of errors found.
/// Call this from `build.rs`.
pub fn scan_project() -> usize {
    slintscanners::scan_project()
}
