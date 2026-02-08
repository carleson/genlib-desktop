//! Genlib Desktop - Dokumenthanteringssystem för släktforskning
//!
//! En native desktop-applikation byggd med Rust och egui.

#![allow(dead_code)]

pub mod models;
pub mod db;
pub mod gedcom;
pub mod services;
pub mod ui;
pub mod utils;

// Re-exports
pub use db::Database;
pub use models::*;
pub use ui::{AppState, View};
