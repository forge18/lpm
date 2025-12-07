// Core functionality
pub mod core;

// Path setup and Lua runner
pub mod path_setup;

// Package manifest
pub mod package;

// Re-export commonly used types
pub use core::{LpmError, LpmResult, CredentialStore, ErrorHelp, format_error_with_help};
pub use path_setup::{LuaRunner, RunOptions, PathSetup};
pub use package::manifest::PackageManifest;

