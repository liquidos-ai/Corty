//! Command popup widget that appears when typing '/' in the chat input
//!
//! This widget displays available slash commands and allows the user to
//! select or filter them. It appears below the input area, pushing the
//! chat history down.

use super::constants::*;
use crate::slash_command::{built_in_slash_commands, SlashCommand};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Widget},
};

/// Maximum number of commands to show in the popup
const MAX_VISIBLE_COMMANDS: usize = 5;

pub(crate) struct CommandPopup {
    /// All available commands
    commands: Vec<SlashCommand>,
    /// Currently filtered commands based on input
    filtered_commands: Vec<SlashCommand>,
    /// Currently selected index in the filtered list
    selected_index: Option<usize>,
    /// Current filter string (without the leading '/')
    filter: String,
}

impl CommandPopup {
    pub(crate) fn new() -> Self {
        let commands = built_in_slash_commands();
        let filtered_commands = commands.clone();
        let selected_index = if filtered_commands.is_empty() {
            None
        } else {
            Some(0)
        };

        Self {
            commands,
            filtered_commands,
            selected_index,
            filter: String::new(),
        }
    }

    /// Update the filter based on the current input text
    pub(crate) fn update_filter(&mut self, text: &str) {
        // Extract the command part after '/'
        let filter = if let Some(stripped) = text.strip_prefix(COMMAND_PREFIX) {
            // Take only the first word (command name)
            stripped.split_whitespace().next().unwrap_or("").to_string()
        } else {
            String::new()
        };

        self.filter = filter;
        self.apply_filter();
    }

    /// Apply the current filter to update the filtered commands list
    fn apply_filter(&mut self) {
        self.filtered_commands = if self.filter.is_empty() {
            self.commands.clone()
        } else {
            self.commands
                .iter()
                .filter(|cmd| {
                    cmd.command()
                        .to_lowercase()
                        .starts_with(&self.filter.to_lowercase())
                })
                .copied()
                .collect()
        };

        // Update selected index
        if self.filtered_commands.is_empty() {
            self.selected_index = None;
        } else {
            // Keep selection within bounds
            self.selected_index = Some(
                self.selected_index
                    .unwrap_or(0)
                    .min(self.filtered_commands.len().saturating_sub(1)),
            );
        }
    }

    /// Move selection up
    pub(crate) fn select_previous(&mut self) {
        if let Some(index) = self.selected_index {
            if index > 0 {
                self.selected_index = Some(index - 1);
            } else {
                self.selected_index = Some(self.filtered_commands.len().saturating_sub(1));
            }
        }
    }

    /// Move selection down
    pub(crate) fn select_next(&mut self) {
        if let Some(index) = self.selected_index {
            if index < self.filtered_commands.len().saturating_sub(1) {
                self.selected_index = Some(index + 1);
            } else {
                self.selected_index = Some(0);
            }
        }
    }

    /// Get the currently selected command
    pub(crate) fn selected_command(&self) -> Option<SlashCommand> {
        self.selected_index
            .and_then(|idx| self.filtered_commands.get(idx).copied())
    }

    /// Calculate the required height for the popup
    pub(crate) fn calculate_height(&self) -> u16 {
        let content_height = self.filtered_commands.len().clamp(1, MAX_VISIBLE_COMMANDS) as u16;

        // Add 2 for borders
        content_height + 2
    }

    /// Check if there are any filtered commands
    pub(crate) fn has_commands(&self) -> bool {
        !self.filtered_commands.is_empty()
    }
}

impl Widget for &CommandPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the list items
        let items: Vec<ListItem> = if self.filtered_commands.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                NO_MATCHING_COMMANDS,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )]))]
        } else {
            self.filtered_commands
                .iter()
                .enumerate()
                .take(MAX_VISIBLE_COMMANDS)
                .map(|(idx, cmd)| {
                    let is_selected = Some(idx) == self.selected_index;

                    let style = if is_selected {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    let line = Line::from(vec![
                        Span::styled(
                            format!("{}{}", COMMAND_PREFIX, cmd.command()),
                            style.fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(COMMAND_SEPARATOR, style.fg(Color::DarkGray)),
                        Span::styled(cmd.description(), style),
                    ]);

                    ListItem::new(line)
                })
                .collect()
        };

        // Create the list widget with a border
        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(COMMAND_POPUP_TITLE)
                .title_style(Style::default().fg(Color::Gray)),
        );

        list.render(area, buf);
    }
}
