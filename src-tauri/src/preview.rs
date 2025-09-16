//! # Preview Window Module
//!
//! Manages preview window state and lifecycle for in-app website preview
//! with user controls for publishing, editing, and syndication.

pub mod state;
pub mod url_builder;
pub mod ipc;
pub mod commands;
pub mod git;
pub mod github;
pub mod github_api;

pub use state::*;