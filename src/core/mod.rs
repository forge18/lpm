//! Core module re-exports.
//!
//! Most core functionality has been moved to `lpm-core`.
//! These are re-exported for backward compatibility.

pub use lpm_core::*;
pub use lpm_core::core::*;

/// Path setup for LPM binary (not Lua paths).
pub mod path_setup;

/// Path module with workspace support.
/// 
/// Re-exports all functions from lpm-core and adds workspace-aware `find_project_root`.
pub mod path;

