//! UI constants and configuration values
//!
//! This module contains all the magic numbers and configuration constants
//! used throughout the TUI to ensure consistent spacing and sizing.

use super::colors;
use ratatui::style::Color;

// ============================================================================
// Application Layout Constants
// ============================================================================

/// Global padding around the entire application content
pub(crate) const APP_PADDING: u16 = 1;

// ============================================================================
// Welcome Widget Constants
// ============================================================================

/// Total height of the welcome widget area
pub(crate) const WELCOME_WIDGET_HEIGHT: u16 = 17;

/// Height of the bordered section within the welcome widget
pub(crate) const WELCOME_WIDGET_BORDER_HEIGHT: u16 = 9;

/// Width of the bordered section within the welcome widget
pub(crate) const WELCOME_WIDGET_BORDER_WIDTH: u16 = 50;

/// Internal padding within the welcome widget border
pub(crate) const WELCOME_WIDGET_PADDING: u16 = 1;

/// Border color for the welcome widget
pub(crate) const WELCOME_WIDGET_BORDER_COLOR: Color = colors::PRIMARY_TEXT;

// ============================================================================
// Chat Widget Constants
// ============================================================================

/// Placeholder text shown in empty chat input
pub(crate) const PLACEHOLDER_TEXT: &str = "> Type a message to ask the AI...";

/// Placeholder text when AI is processing
pub(crate) const PLACEHOLDER_TEXT_AI_PROCESSING: &str = "AI is processing... Please wait...";

/// Placeholder text for Ask AI command mode
pub(crate) const PLACEHOLDER_TEXT_ASK_AI: &str = "Type your question for the AI assistant...";

/// Mouse capture status indicators
pub(crate) const MOUSE_STATUS_ON: &str = "[ON]";
pub(crate) const MOUSE_STATUS_OFF: &str = "[OFF]";

/// Fullscreen mode indicator
pub(crate) const FULLSCREEN_INDICATOR: &str = "[FULLSCREEN] ";

/// Chat widget border titles
pub(crate) const CHAT_TITLE_AI_PROCESSING: &str = "AI is processing... Input disabled";
pub(crate) const CHAT_TITLE_ASK_AI_MODE: &str =
    "Ask AI mode - Type your question and press Enter | Esc to cancel";
pub(crate) const CHAT_TITLE_NORMAL: &str =
    "Enter to ask AI | Ctrl+d to quit | Ctrl+n for newline | Ctrl+y toggle mouse capture";

// ============================================================================
// Chat History Constants
// ============================================================================

/// User message prefix
pub(crate) const USER_MESSAGE_PREFIX: &str = "> ";

/// Agent message prefix
pub(crate) const AGENT_MESSAGE_PREFIX: &str = "● ";

/// AI working indicator base text
pub(crate) const AI_WORKING_TEXT: &str = "● AI is thinking";

/// Chat history title formats
pub(crate) const CHAT_HISTORY_TITLE_FOCUSED: &str = "Messages (↑/↓ or j/k = line,  b/space = page)";
pub(crate) const CHAT_HISTORY_TITLE_UNFOCUSED: &str = "Messages";

/// Scrollbar symbols
pub(crate) const SCROLLBAR_UP_SYMBOL: &str = "↑";
pub(crate) const SCROLLBAR_DOWN_SYMBOL: &str = "↓";
pub(crate) const SCROLLBAR_THUMB_SYMBOL: &str = "█";
pub(crate) const SCROLLBAR_TRACK_SYMBOL: &str = "│";

// ============================================================================
// Command Popup Constants
// ============================================================================

/// Command popup title
pub(crate) const COMMAND_POPUP_TITLE: &str = "Commands";

/// No matching commands message
pub(crate) const NO_MATCHING_COMMANDS: &str = "No matching commands";

/// Command format
pub(crate) const COMMAND_PREFIX: &str = "/";
pub(crate) const COMMAND_SEPARATOR: &str = " - ";

// ============================================================================
// Slash Command Constants
// ============================================================================

/// Slash command descriptions
pub(crate) const SLASH_COMMAND_CLEAR_DESC: &str = "Clear the chat history";
pub(crate) const SLASH_COMMAND_ASK_AI_DESC: &str =
    "Ask the AI (same as typing without slash command)";
pub(crate) const SLASH_COMMAND_FULLSCREEN_DESC: &str =
    "Toggle fullscreen mode (hide/show welcome widget)";

// ============================================================================
// Error Messages
// ============================================================================

/// Log error messages
pub(crate) const ERROR_TOGGLE_MOUSE_MODE: &str = "Failed to toggle mouse mode: {}";

/// AI error messages
pub(crate) const AI_ERROR_CONNECTION_TIMEOUT: &str = "AI Service Error: Connection timeout - Unable to reach the AI backend. Please check your network connection and try again.";
pub(crate) const AI_ERROR_RESPONSE: &str =
    "Sorry, I couldn't process your request due to a connection error. Please try again later.";

// ============================================================================
// Toaster Constants
// ============================================================================

/// Toast icons
pub(crate) const TOAST_ICON_ERROR: &str = "✕";
pub(crate) const TOAST_ICON_WARNING: &str = "⚠";
pub(crate) const TOAST_ICON_INFO: &str = "ℹ";
pub(crate) const TOAST_ICON_SUCCESS: &str = "✓";

/// Toast titles
pub(crate) const TOAST_TITLE_ERROR: &str = "Error";
pub(crate) const TOAST_TITLE_WARNING: &str = "Warning";
pub(crate) const TOAST_TITLE_INFO: &str = "Info";
pub(crate) const TOAST_TITLE_SUCCESS: &str = "Success";

// ============================================================================
// Welcome Widget Constants
// ============================================================================

/// Welcome widget texts
pub(crate) const WELCOME_TEXT_PREFIX: &str = "* Welcome to ";
pub(crate) const WELCOME_TEXT_APP_NAME: &str = "Corty Code";
pub(crate) const WELCOME_TEXT_SUFFIX: &str = "!";
pub(crate) const WELCOME_TEXT_INSTRUCTIONS: &str = "Just type to ask the AI, or use / for commands";
pub(crate) const WELCOME_TEXT_CWD_PREFIX: &str = "cwd: ";
pub(crate) const WELCOME_TEXT_CWD_UNKNOWN: &str = "(unknown)";

/// Welcome widget help tips
pub(crate) const WELCOME_TIPS_HEADER: &str = "Tips for getting started:";
pub(crate) const WELCOME_TIP_1: &str = "1. Type your message and press Enter to ask the AI";
pub(crate) const WELCOME_TIP_2: &str = "2. Use /clear to clear the chat history";
pub(crate) const WELCOME_TIP_3: &str = "3. Use Ctrl+N to add newlines to your message";
pub(crate) const WELCOME_TIP_4: &str = "4. Use Ctrl+Y to toggle mouse capture mode";

// ============================================================================
// Animation Constants
// ============================================================================

/// AI working animation dot character
pub(crate) const AI_WORKING_DOT: &str = ".";

/// Animation timing (milliseconds)
pub(crate) const AI_WORKING_DOT_INTERVAL: u64 = 500;
pub(crate) const AI_WORKING_DOT_MAX_COUNT: u64 = 4;
