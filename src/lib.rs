#[macro_use]
extern crate tracing;

mod action;
mod args;
pub mod cargo;
mod component;
mod components;
pub mod logging;
mod mermaid;
pub mod metadata;
pub mod run;
pub mod tui;

pub use args::Args;
pub use components::app::App;
pub use logging::initialize_logging;
pub use run::run_loop;
