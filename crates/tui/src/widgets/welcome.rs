//! Welcome widget displaying application header and helpful information
//!
//! This widget shows:
//! - Application title and version
//! - Current working directory
//! - Getting started tips

use super::{colors, constants::*};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};
use std::env;

/// Welcome widget that displays the application header and help text
pub(crate) struct WelcomeWidget {
    /// Current working directory to display
    cwd: String,
}

impl WelcomeWidget {
    /// Create a new welcome widget with the current working directory
    pub fn new() -> Self {
        let cwd = env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from(WELCOME_TEXT_CWD_UNKNOWN));

        Self { cwd }
    }

    /// Render the welcome widget content
    fn render_content(self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(WELCOME_WIDGET_BORDER_WIDTH),
                Constraint::Min(0),
            ])
            .split(area);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(WELCOME_WIDGET_BORDER_HEIGHT),
                Constraint::Min(0),
            ])
            .split(horizontal[0]);

        // Build welcome message text
        let welcome_text = Text::from(vec![
            Line::from(vec![
                Span::raw(WELCOME_TEXT_PREFIX),
                Span::raw(WELCOME_TEXT_APP_NAME)
                    .fg(colors::PRIMARY_TEXT)
                    .bold(),
                Span::raw(WELCOME_TEXT_SUFFIX),
            ]),
            Line::raw(""),
            Line::from(
                Span::raw(WELCOME_TEXT_INSTRUCTIONS).style(Style::default().fg(colors::DARK_GRAY)),
            ),
            Line::raw(""),
            Line::from(vec![
                Span::raw(WELCOME_TEXT_CWD_PREFIX).fg(colors::DARK_GRAY),
                Span::raw(self.cwd).fg(colors::DARK_GRAY),
            ]),
        ]);

        // Create bordered welcome paragraph
        let welcome_paragraph = Paragraph::new(welcome_text)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(WELCOME_WIDGET_BORDER_COLOR))
                    .padding(Padding::uniform(WELCOME_WIDGET_PADDING)),
            );

        // Build help tips text
        let help_tips = vec![
            Line::from(WELCOME_TIPS_HEADER).fg(colors::DARK_GRAY),
            Line::raw(""),
            Line::from(WELCOME_TIP_1).fg(colors::DARK_GRAY),
            Line::from(WELCOME_TIP_2).fg(colors::DARK_GRAY),
            Line::from(WELCOME_TIP_3).fg(colors::DARK_GRAY),
            Line::from(WELCOME_TIP_4).fg(colors::DARK_GRAY),
        ];
        let help_text = Text::from(help_tips);

        // Create help paragraph
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Left);

        // Render both sections
        welcome_paragraph.render(vertical[0], buf);
        help_paragraph.render(vertical[1], buf);
    }
}

impl Widget for WelcomeWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.render_content(area, buf);
    }
}
