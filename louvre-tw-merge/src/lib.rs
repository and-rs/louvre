#![forbid(unsafe_code)]

//! Tailwind CSS class merging vendored from `tw_merge` 0.1.21.

mod ast;
mod core;

pub use crate::core::{AsTailwindClass, merge};

/// Merges Tailwind class lists, preserving non-conflicting classes and giving
/// the rightmost conflicting utility precedence.
pub fn merge_classes(parts: &[&str]) -> String {
    core::merge::tw_merge_slice(parts)
}
