pub mod discovery;
pub mod renderer;
pub mod metadata;
pub mod commands;

pub use discovery::TemplateDiscovery;
pub use renderer::TemplateRenderer;
pub use commands::{TemplateCommands, run};

