//! Slash commands for the Corty TUI
//!
//! This module defines the available slash commands that can be invoked
//! by typing '/' followed by the command name in the chat input.

use crate::widgets::constants::{
    SLASH_COMMAND_ASK_AI_DESC, SLASH_COMMAND_CLEAR_DESC, SLASH_COMMAND_FULLSCREEN_DESC,
};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString, IntoStaticStr};

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    /// Clear the chat history
    Clear,
    /// Ask the AI assistant for help
    AskAI,
    /// Toggle fullscreen mode
    Fullscreen,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Clear => SLASH_COMMAND_CLEAR_DESC,
            SlashCommand::AskAI => SLASH_COMMAND_ASK_AI_DESC,
            SlashCommand::Fullscreen => SLASH_COMMAND_FULLSCREEN_DESC,
        }
    }

    /// Command string without the leading '/'.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Returns true if this command requires additional input after selection
    pub fn requires_input(self) -> bool {
        match self {
            SlashCommand::Clear => false,
            SlashCommand::AskAI => true,
            SlashCommand::Fullscreen => false,
        }
    }
}

/// Return all built-in commands as a vector.
pub fn built_in_slash_commands() -> Vec<SlashCommand> {
    SlashCommand::iter().collect()
}
