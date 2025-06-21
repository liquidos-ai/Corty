//! Widget components for the Corty TUI
//!
//! This module contains all the UI widgets used in the application:
//! - Chat widget for user input
//! - Chat history for message display
//! - Welcome widget for the application header
//! - Various utilities for text rendering and styling

mod chat;
mod chat_history;
mod colors;
mod command_popup;
pub(crate) mod constants;
mod text_block;
mod toaster;
mod welcome;

// Re-export commonly used items
pub(crate) use chat::{ChatWidget, ChatWidgetState};
pub use toaster::{Toaster, ToasterState};
pub(crate) use welcome::WelcomeWidget;
