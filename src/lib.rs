//! Lupine: reusable, offline terminal interactive fiction engine.
pub mod commands;
pub mod engine;
pub mod parser;
pub mod render;
pub mod runner;
pub mod state;
pub mod world;

pub use runner::run;
