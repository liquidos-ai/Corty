//! Tests for slash command functionality

#[cfg(test)]
mod tests {
    use corty_tui::slash_command::{built_in_slash_commands, SlashCommand};

    #[test]
    fn test_slash_commands_available() {
        let commands = built_in_slash_commands();
        assert_eq!(commands.len(), 3);
        assert!(commands.contains(&SlashCommand::Clear));
        assert!(commands.contains(&SlashCommand::AskAI));
        assert!(commands.contains(&SlashCommand::Fullscreen));
    }

    #[test]
    fn test_slash_command_descriptions() {
        assert_eq!(SlashCommand::Clear.description(), "Clear the chat history");
        assert_eq!(
            SlashCommand::AskAI.description(),
            "Ask the AI (same as typing without slash command)"
        );
        assert_eq!(
            SlashCommand::Fullscreen.description(),
            "Toggle fullscreen mode (hide/show welcome widget)"
        );
    }

    #[test]
    fn test_slash_command_strings() {
        assert_eq!(SlashCommand::Clear.command(), "clear");
        assert_eq!(SlashCommand::AskAI.command(), "ask-ai");
        assert_eq!(SlashCommand::Fullscreen.command(), "fullscreen");
    }

    #[test]
    fn test_slash_command_requires_input() {
        assert_eq!(SlashCommand::Clear.requires_input(), false);
        assert_eq!(SlashCommand::AskAI.requires_input(), true);
        assert_eq!(SlashCommand::Fullscreen.requires_input(), false);
    }
}
