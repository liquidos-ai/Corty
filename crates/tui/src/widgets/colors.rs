//! Color definitions for the TUI theme
//!
//! This module contains all the color constants used throughout the TUI
//! to maintain a consistent visual theme.

use ratatui::style::Color;

/// Primary text color - a warm red tone for important elements
pub(crate) const PRIMARY_TEXT: Color = Color::Rgb(248, 113, 113);

/// Dark gray color for secondary/dimmed text and UI elements
pub(crate) const DARK_GRAY: Color = Color::Rgb(140, 140, 140);
