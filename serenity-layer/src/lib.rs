mod api;
mod commands;

pub type Error = Box<dyn std::error::Error + Send + Sync>;


// Export a prelude
pub mod prelude {
    pub use crate::api::*;
    pub use crate::commands::*;
}