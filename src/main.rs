//! Genlib Desktop - Entry Point
//!
//! Ett dokumenthanteringssystem för släktforskning.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod app;
mod db;
mod gedcom;
mod models;
mod services;
mod ui;
mod utils;

use app::GenlibApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initiera logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("Startar Genlib Desktop v{}", env!("CARGO_PKG_VERSION"));

    // Fönsterinställningar
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("Genlib - Släktforskning v{}", env!("CARGO_PKG_VERSION")))
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    // Starta applikationen
    eframe::run_native(
        "Genlib",
        options,
        Box::new(|cc| Ok(Box::new(GenlibApp::new(cc)))),
    )
}
