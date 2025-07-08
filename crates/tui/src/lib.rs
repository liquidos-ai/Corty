//! Corty TUI - Terminal User Interface for Corty
//!
//! This crate provides a terminal-based user interface for interacting with
//! the Corty AI assistant. It features:
//!
//! - Interactive chat interface with message history
//! - Mouse and keyboard navigation support
//! - Configurable mouse capture for scrolling vs text selection
//! - Responsive layout with welcome screen
//! - Real-time message updates and AI responses

use color_eyre::eyre::Result;

mod app;
mod event;
pub mod slash_command;
mod tui;
mod utils;

//pub so that it can be imported in test modules
pub mod widgets;

/// Run the terminal user interface application
///
/// This is the main entry point for the TUI. It initializes the terminal,
/// sets up the application state, and runs the main event loop.
///
/// # Errors
///
/// Returns an error if:
/// - Terminal initialization fails
/// - The event loop encounters an unrecoverable error
/// - Terminal restoration fails
pub async fn run_tui() -> Result<()> {
    // Initialize terminal and mouse capture
    let (mut terminal, mut mouse_capture) = tui::init()?;

    // Create and run the application
    let mut app = app::App::new().await;
    app.run(&mut terminal, &mut mouse_capture).await?;

    Ok(())
}
