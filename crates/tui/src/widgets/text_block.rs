//! Simple text block widget for rendering multi-line text
//!
//! This widget provides a lightweight wrapper around Paragraph for
//! rendering pre-formatted text blocks in the chat history.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Paragraph, Widget},
};

/// A simple widget that displays a list of text lines
///
/// This is used internally by the chat history to render message content
/// without the overhead of full paragraph features like wrapping or scrolling.
#[derive(Clone)]
pub(crate) struct TextBlock {
    /// The lines of text to display
    pub(crate) lines: Vec<Line<'static>>,
}

impl TextBlock {
    /// Create a new text block with the given lines
    pub(crate) fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl Widget for TextBlock {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Delegate to Paragraph for actual rendering
        Paragraph::new(self.lines).render(area, buf);
    }
}
