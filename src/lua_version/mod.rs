pub mod detector;
pub mod constraint;
pub mod compatibility;

pub use detector::{LuaVersionDetector, LuaVersion};
pub use constraint::{LuaVersionConstraint, parse_lua_version_constraint};
pub use compatibility::PackageCompatibility;

