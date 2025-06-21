//! Toaster widget for displaying temporary notifications
//!
//! This widget shows error messages and other notifications that appear
//! temporarily and then fade away. Messages are stacked vertically and
//! can be dismissed manually or automatically after a timeout.

use super::constants::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use std::time::{Duration, Instant};

/// Maximum number of toast messages to display simultaneously
const MAX_TOASTS: usize = 3;

/// How long a toast message stays visible before auto-dismissing
const TOAST_DURATION: Duration = Duration::from_secs(5);

/// A single toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    /// The message to display
    pub message: String,
    /// The type of toast (affects styling)
    pub kind: ToastKind,
    /// When this toast was created
    pub created_at: Instant,
}

/// Types of toast notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// Error messages (red)
    Error,
    /// Warning messages (yellow)
    #[allow(dead_code)]
    Warning,
    /// Info messages (blue)
    #[allow(dead_code)]
    Info,
    /// Success messages (green)
    #[allow(dead_code)]
    Success,
}

impl ToastKind {
    /// Get the color for this toast type
    fn color(&self) -> Color {
        match self {
            ToastKind::Error => Color::Rgb(239, 68, 68),    // Red
            ToastKind::Warning => Color::Rgb(245, 158, 11), // Yellow
            ToastKind::Info => Color::Rgb(59, 130, 246),    // Blue
            ToastKind::Success => Color::Rgb(34, 197, 94),  // Green
        }
    }

    /// Get the icon for this toast type
    fn icon(&self) -> &'static str {
        match self {
            ToastKind::Error => TOAST_ICON_ERROR,
            ToastKind::Warning => TOAST_ICON_WARNING,
            ToastKind::Info => TOAST_ICON_INFO,
            ToastKind::Success => TOAST_ICON_SUCCESS,
        }
    }

    /// Get the title for this toast type
    fn title(&self) -> &'static str {
        match self {
            ToastKind::Error => TOAST_TITLE_ERROR,
            ToastKind::Warning => TOAST_TITLE_WARNING,
            ToastKind::Info => TOAST_TITLE_INFO,
            ToastKind::Success => TOAST_TITLE_SUCCESS,
        }
    }
}

/// State for managing toast notifications
pub struct ToasterState {
    /// Active toast messages
    toasts: Vec<Toast>,
}

impl Default for ToasterState {
    fn default() -> Self {
        Self::new()
    }
}

impl ToasterState {
    /// Create a new toaster state
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    /// Add a new toast message
    pub fn add_toast(&mut self, message: String, kind: ToastKind) {
        let toast = Toast {
            message,
            kind,
            created_at: Instant::now(),
        };

        self.toasts.push(toast);

        // Keep only the most recent MAX_TOASTS
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.drain(0..self.toasts.len() - MAX_TOASTS);
        }
    }

    /// Add an error toast
    pub fn error(&mut self, message: impl Into<String>) {
        self.add_toast(message.into(), ToastKind::Error);
    }

    /// Add a warning toast
    #[allow(dead_code)]
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add_toast(message.into(), ToastKind::Warning);
    }

    /// Add an info toast
    #[allow(dead_code)]
    pub fn info(&mut self, message: impl Into<String>) {
        self.add_toast(message.into(), ToastKind::Info);
    }

    /// Add a success toast
    #[allow(dead_code)]
    pub fn success(&mut self, message: impl Into<String>) {
        self.add_toast(message.into(), ToastKind::Success);
    }

    /// Remove expired toasts
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.toasts
            .retain(|toast| now.duration_since(toast.created_at) < TOAST_DURATION);
    }

    /// Check if there are any active toasts
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// Get the active toasts
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    /// Clear all toasts
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.toasts.clear();
    }
}

/// Widget for rendering toast notifications
pub struct Toaster<'a> {
    state: &'a ToasterState,
}

impl<'a> Toaster<'a> {
    /// Create a new toaster widget
    pub fn new(state: &'a ToasterState) -> Self {
        Self { state }
    }

    /// Calculate the position for toasts (top-right corner with padding)
    fn calculate_position(&self, container: Rect) -> (u16, u16) {
        let toast_width = self.calculate_width();
        let x = container.x + container.width.saturating_sub(toast_width + 2);
        let y = container.y + 2;
        (x, y)
    }

    /// Calculate the width needed for toasts
    fn calculate_width(&self) -> u16 {
        // Find the longest message
        let max_message_len = self
            .state
            .toasts()
            .iter()
            .map(|t| t.message.len())
            .max()
            .unwrap_or(0);

        // Add space for icon, title, borders, and padding
        (max_message_len + 20).min(60) as u16
    }

    /// Calculate the height of a single toast
    fn calculate_toast_height(&self, toast: &Toast, width: u16) -> u16 {
        // Account for wrapping - very simple estimation
        let content_width = width.saturating_sub(4) as usize; // borders and padding
        let lines = toast.message.len().div_ceil(content_width);
        // 1 line for title, N lines for message, 2 for borders
        (lines as u16 + 3).min(6)
    }
}

impl<'a> Widget for Toaster<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.has_toasts() {
            return;
        }

        let (x, mut y) = self.calculate_position(area);
        let width = self.calculate_width();

        // Render each toast
        for (idx, toast) in self.state.toasts().iter().enumerate() {
            let height = self.calculate_toast_height(toast, width);

            // Make sure we don't render outside the area
            if y + height > area.y + area.height {
                break;
            }

            let toast_area = Rect {
                x,
                y,
                width,
                height,
            };

            // Calculate opacity based on age and position
            let age_factor = {
                let elapsed = toast.created_at.elapsed();
                if elapsed >= TOAST_DURATION.mul_f32(0.8) {
                    // Start fading in the last 20% of duration
                    let fade_progress = (elapsed.as_secs_f32()
                        - TOAST_DURATION.as_secs_f32() * 0.8)
                        / (TOAST_DURATION.as_secs_f32() * 0.2);
                    1.0 - fade_progress.min(1.0)
                } else {
                    1.0
                }
            };

            // Older toasts are more transparent
            let position_factor = 1.0 - (idx as f32 * 0.2);
            let opacity_factor = (age_factor * position_factor).max(0.3);

            // Create the toast content
            let color = toast.kind.color();
            let dimmed_color = if opacity_factor < 0.8 {
                Color::Rgb(
                    (color.r() as f32 * opacity_factor) as u8,
                    (color.g() as f32 * opacity_factor) as u8,
                    (color.b() as f32 * opacity_factor) as u8,
                )
            } else {
                color
            };

            // Create title line
            let title_line = Line::from(vec![
                Span::styled(
                    format!("{} ", toast.kind.icon()),
                    Style::default()
                        .fg(dimmed_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    toast.kind.title(),
                    Style::default()
                        .fg(dimmed_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);

            // Create message lines
            let message_lines: Vec<Line> = toast
                .message
                .lines()
                .map(|line| {
                    Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Rgb(200, 200, 200)),
                    ))
                })
                .collect();

            // Combine all lines
            let mut content = vec![title_line];
            content.extend(message_lines);

            // Create the toast block
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(dimmed_color))
                .style(Style::default().bg(Color::Rgb(20, 20, 20)));

            // Render the toast
            let paragraph = Paragraph::new(content)
                .block(block)
                .alignment(Alignment::Left);

            paragraph.render(toast_area, buf);

            // Move down for next toast
            y += height + 1;
        }
    }
}

// Helper trait to get RGB values from Color
trait ColorRgb {
    fn r(&self) -> u8;
    fn g(&self) -> u8;
    fn b(&self) -> u8;
}

impl ColorRgb for Color {
    fn r(&self) -> u8 {
        match self {
            Color::Rgb(r, _, _) => *r,
            _ => 0,
        }
    }

    fn g(&self) -> u8 {
        match self {
            Color::Rgb(_, g, _) => *g,
            _ => 0,
        }
    }

    fn b(&self) -> u8 {
        match self {
            Color::Rgb(_, _, b) => *b,
            _ => 0,
        }
    }
}
