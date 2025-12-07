pub mod compatibility;
pub mod constraint;
pub mod detector;

pub use compatibility::PackageCompatibility;
pub use constraint::{parse_lua_version_constraint, LuaVersionConstraint};
pub use detector::{LuaVersion, LuaVersionDetector};
