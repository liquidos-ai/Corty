use super::constants::*;
use super::text_block::TextBlock;
use crate::event::AppEventSender;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, StatefulWidget,
    },
};
use std::cell::Cell;

/// Common wrap configuration for consistent text measurement and rendering
#[inline]
const fn wrap_config() -> ratatui::widgets::Wrap {
    ratatui::widgets::Wrap { trim: false }
}

/// A single history entry with cached line count for performance
struct Entry {
    cell: ChatHistoryBlock,
    line_count: Cell<usize>,
}

/// Types of messages that can appear in the chat history
pub(crate) enum ChatHistoryBlock {
    UserMessage { view: TextBlock },
    AgentMessage { view: TextBlock },
    AiWorking { elapsed_ms: u64 },
}

impl ChatHistoryBlock {
    pub(crate) fn new_ai_working() -> Self {
        ChatHistoryBlock::AiWorking { elapsed_ms: 0 }
    }

    pub(crate) fn new_user_message(message: String) -> Self {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut message_lines = message.lines();

        if let Some(first_line) = message_lines.next() {
            lines.push(Line::from(format!("{}{}", USER_MESSAGE_PREFIX, first_line)));
        }
        for line in message_lines {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines.push(Line::from(""));

        ChatHistoryBlock::UserMessage {
            view: TextBlock::new(lines),
        }
    }

    pub(crate) fn new_agent_message(message: String) -> Self {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut message_lines = message.lines();

        if let Some(first_line) = message_lines.next() {
            lines.push(Line::from(format!(
                "{}{}",
                AGENT_MESSAGE_PREFIX, first_line
            )));
        }
        for line in message_lines {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines.push(Line::from(""));

        ChatHistoryBlock::AgentMessage {
            view: TextBlock::new(lines),
        }
    }

    /// Calculate the height of the block when rendered at a specific width
    fn height(&self, width: u16) -> usize {
        match self {
            ChatHistoryBlock::UserMessage { view } | ChatHistoryBlock::AgentMessage { view } => {
                // Early return for edge cases
                if width == 0 || view.lines.is_empty() {
                    return 1;
                }

                let mut total_lines = 0;
                let width_usize = width as usize;

                // Performance optimization for very large messages
                const MAX_LINES_EXACT: usize = 500;
                const MAX_TOTAL_LINES: usize = 5000;

                if view.lines.len() > MAX_LINES_EXACT {
                    return (view.lines.len() * 2).min(MAX_TOTAL_LINES);
                }

                for line in &view.lines {
                    let line_width = line.width();
                    if line_width == 0 {
                        total_lines += 1; // Empty line
                    } else if line_width <= width_usize {
                        total_lines += 1; // Fits on one line
                    } else {
                        // Calculate wrapped lines with bounds checking
                        let wrapped_lines = line_width.div_ceil(width_usize);
                        total_lines += wrapped_lines.min(100); // Cap per-line wrapping
                    }

                    // Performance cap
                    const MAX_PROCESSED_LINES: usize = 1000;
                    if total_lines > MAX_PROCESSED_LINES {
                        return MAX_PROCESSED_LINES;
                    }
                }
                total_lines.clamp(1, 1000)
            }
            ChatHistoryBlock::AiWorking { .. } => 2, // AI working indicator takes 2 lines
        }
    }

    /// Render the block into a buffer with vertical scroll offset
    fn render_window(&self, skip_lines: usize, area: Rect, buf: &mut Buffer) {
        match self {
            ChatHistoryBlock::UserMessage { view } | ChatHistoryBlock::AgentMessage { view } => {
                let paragraph = Paragraph::new(view.lines.clone())
                    .wrap(wrap_config())
                    .scroll((skip_lines as u16, 0));
                paragraph.render(area, buf);
            }
            ChatHistoryBlock::AiWorking { elapsed_ms } => {
                // Create animated dots based on elapsed time
                let dots_count = (elapsed_ms / AI_WORKING_DOT_INTERVAL) % AI_WORKING_DOT_MAX_COUNT;
                let dots = AI_WORKING_DOT.repeat(dots_count as usize);
                let working_text = format!("{}{}", AI_WORKING_TEXT, dots);

                let lines = vec![
                    Line::from(vec![Span::styled(
                        working_text,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::ITALIC),
                    )]),
                    Line::from(""),
                ];

                let paragraph = Paragraph::new(lines)
                    .wrap(wrap_config())
                    .scroll((skip_lines as u16, 0));
                paragraph.render(area, buf);
            }
        }
    }
}

/// State management for the chat history widget
pub struct ChatHistoryState {
    entries: Vec<Entry>,
    /// Width used for line count calculations
    cached_width: Cell<u16>,
    /// Current scroll position (usize::MAX = stick to bottom)
    scroll_position: usize,
    /// Cached number of rendered lines
    num_rendered_lines: Cell<usize>,
    /// Cached viewport height
    last_viewport_height: Cell<usize>,
    /// Whether this widget has input focus
    has_input_focus: bool,
    /// Current mouse capture state
    mouse_capture_active: bool,
}

impl Default for ChatHistoryState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            cached_width: Cell::new(0),
            scroll_position: usize::MAX, // Start in "stick to bottom" mode
            num_rendered_lines: Cell::new(0),
            last_viewport_height: Cell::new(0),
            has_input_focus: false,
            mouse_capture_active: true,
        }
    }
}

impl ChatHistoryState {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn set_input_focus(&mut self, has_input_focus: bool) {
        self.has_input_focus = has_input_focus;
    }

    pub fn set_mouse_capture_active(&mut self, is_active: bool) {
        self.mouse_capture_active = is_active;
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll_position = usize::MAX;
        self.num_rendered_lines.set(0);
        self.last_viewport_height.set(0);
    }

    pub fn add_user_message(&mut self, message: String) {
        self.add_to_history(ChatHistoryBlock::new_user_message(message));
    }

    pub fn add_agent_message(&mut self, message: String) {
        self.add_to_history(ChatHistoryBlock::new_agent_message(message));
    }

    fn add_to_history(&mut self, cell: ChatHistoryBlock) {
        let width = self.cached_width.get();
        let count = if width > 0 { cell.height(width) } else { 0 };

        self.entries.push(Entry {
            cell,
            line_count: Cell::new(count),
        });
        // Adding a new message pins the scroll to the bottom
        self.scroll_position = usize::MAX;
    }

    pub fn add_ai_working(&mut self) {
        self.add_to_history(ChatHistoryBlock::new_ai_working());
    }

    pub fn remove_ai_working(&mut self) {
        // Remove the AiWorking indicator if it exists
        self.entries
            .retain(|entry| !matches!(entry.cell, ChatHistoryBlock::AiWorking { .. }));
    }

    pub fn update_ai_working(&mut self, elapsed_ms: u64) {
        // Update the elapsed time for AI working indicator
        for entry in &mut self.entries {
            if let ChatHistoryBlock::AiWorking {
                elapsed_ms: ref mut ms,
            } = entry.cell
            {
                *ms = elapsed_ms;
            }
        }
    }

    #[allow(dead_code)]
    pub fn has_ai_working(&self) -> bool {
        // Check if there's an AI working indicator in the entries
        self.entries
            .iter()
            .any(|entry| matches!(entry.cell, ChatHistoryBlock::AiWorking { .. }))
    }
}

/// Widget for rendering scrollable chat history
pub(crate) struct ChatHistory {
    #[allow(dead_code)]
    app_event_tx: AppEventSender,
}

impl ChatHistory {
    pub fn new(app_event_tx: AppEventSender) -> Self {
        Self { app_event_tx }
    }

    pub fn scroll(&self, delta: i32, state: &mut ChatHistoryState) {
        match delta.cmp(&0) {
            std::cmp::Ordering::Less => self.scroll_up(-delta as u32, state),
            std::cmp::Ordering::Greater => self.scroll_down(delta as u32, state),
            std::cmp::Ordering::Equal => {}
        }
    }

    fn scroll_up(&self, num_lines: u32, state: &mut ChatHistoryState) {
        // Convert from "stick to bottom" mode to explicit position if needed
        if state.scroll_position == usize::MAX {
            state.scroll_position = state
                .num_rendered_lines
                .get()
                .saturating_sub(state.last_viewport_height.get());
        }

        state.scroll_position = state.scroll_position.saturating_sub(num_lines as usize);
    }

    fn scroll_down(&self, num_lines: u32, state: &mut ChatHistoryState) {
        // Early return if already at bottom
        if state.scroll_position == usize::MAX {
            return;
        }

        let viewport_height = state.last_viewport_height.get().max(1);
        let num_rendered_lines = state.num_rendered_lines.get();

        // Calculate maximum scroll position
        let max_scroll = num_rendered_lines.saturating_sub(viewport_height);

        let new_pos = state.scroll_position.saturating_add(num_lines as usize);

        if new_pos >= max_scroll {
            // Switch to automatic stick-to-bottom mode
            state.scroll_position = usize::MAX;
        } else {
            state.scroll_position = new_pos;
        }
    }

    /// Handle keyboard events for scrolling. Returns true if redraw needed.
    pub(crate) fn handle_key_event(
        &self,
        key_event: KeyEvent,
        state: &mut ChatHistoryState,
    ) -> bool {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_up(1, state);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_down(1, state);
                true
            }
            KeyCode::PageUp | KeyCode::Char('b') => {
                self.scroll_page_up(state);
                true
            }
            KeyCode::PageDown | KeyCode::Char(' ') => {
                self.scroll_page_down(state);
                true
            }
            _ => false,
        }
    }

    fn scroll_page_up(&self, state: &mut ChatHistoryState) {
        let viewport_height = state.last_viewport_height.get().max(1);

        // Convert from implicit to explicit scroll position if needed
        if state.scroll_position == usize::MAX {
            state.scroll_position = state
                .num_rendered_lines
                .get()
                .saturating_sub(viewport_height);
        }

        // Move up by viewport height
        state.scroll_position = state.scroll_position.saturating_sub(viewport_height);
    }

    fn scroll_page_down(&self, state: &mut ChatHistoryState) {
        // Early return if already at bottom
        if state.scroll_position == usize::MAX {
            return;
        }

        let viewport_height = state.last_viewport_height.get().max(1);
        let num_lines = state.num_rendered_lines.get();

        // Calculate maximum valid scroll position
        let max_scroll = num_lines.saturating_sub(viewport_height);

        // Move down by viewport height
        let new_pos = state.scroll_position.saturating_add(viewport_height);

        if new_pos >= max_scroll {
            // Switch back to automatic stick-to-bottom mode
            state.scroll_position = usize::MAX;
        } else {
            state.scroll_position = new_pos;
        }
    }
}

impl ChatHistory {
    /// Render the chat history with proper scrolling and focus indication
    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &mut ChatHistoryState) {
        let mouse_indicator = if state.mouse_capture_active {
            MOUSE_STATUS_ON.replace("[", "[Mouse ")
        } else {
            MOUSE_STATUS_OFF.replace("[", "[Mouse ")
        };

        let (title, border_style) = if state.has_input_focus {
            (
                format!("{} {}", CHAT_HISTORY_TITLE_FOCUSED, mouse_indicator),
                Style::default().fg(Color::LightYellow),
            )
        } else {
            (
                format!("{} {}", CHAT_HISTORY_TITLE_UNFOCUSED, mouse_indicator),
                Style::default().dim(),
            )
        };

        let block = Block::default()
            .title(title.as_str())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style);

        let inner = block.inner(area);
        Clear.render(area, buf);
        block.render(area, buf);

        // Reserve space for scrollbar
        let effective_width = inner.width.saturating_sub(1);
        if effective_width == 0 {
            return;
        }

        // Recalculate line counts if width has changed
        let num_lines: usize = if state.cached_width.get() != effective_width {
            state.cached_width.set(effective_width);
            let mut num_lines: usize = 0;

            // Performance cap for very large histories
            const MAX_ENTRIES_TO_PROCESS: usize = 1000;
            let max_entries = state.entries.len().min(MAX_ENTRIES_TO_PROCESS);
            for entry in state.entries.iter().take(max_entries) {
                let count = entry.cell.height(effective_width);
                num_lines = num_lines.saturating_add(count);
                entry.line_count.set(count);

                // Performance safeguard
                const MAX_RENDERED_LINES: usize = 10000;
                if num_lines > MAX_RENDERED_LINES {
                    break;
                }
            }
            const MAX_RENDERED_LINES: usize = 10000;
            num_lines.min(MAX_RENDERED_LINES)
        } else {
            // Use cached values with performance limit
            const CACHE_PROCESS_LIMIT: usize = 1000;
            let cached_sum = state
                .entries
                .iter()
                .take(CACHE_PROCESS_LIMIT)
                .map(|e| e.line_count.get())
                .sum::<usize>();
            cached_sum.min(10000)
        };

        let viewport_height = inner.height as usize;
        let max_scroll = num_lines.saturating_sub(viewport_height);
        let scroll_pos = if state.scroll_position == usize::MAX {
            max_scroll
        } else {
            state.scroll_position.min(max_scroll)
        };

        // Skip rendering if empty
        if num_lines == 0 || viewport_height == 0 {
            state.num_rendered_lines.set(num_lines);
            state.last_viewport_height.set(viewport_height);
            return;
        }

        // Render visible entries
        let mut y_cursor = inner.y;
        let mut remaining_height = inner.height as usize;
        let mut lines_to_skip = scroll_pos;

        for entry in &state.entries {
            let cell_height = entry.line_count.get();

            if lines_to_skip >= cell_height {
                lines_to_skip -= cell_height;
                continue;
            }

            let visible_height = (cell_height - lines_to_skip).min(remaining_height);
            if visible_height == 0 {
                break;
            }

            let cell_rect = Rect {
                x: inner.x,
                y: y_cursor,
                width: effective_width,
                height: visible_height as u16,
            };
            entry.cell.render_window(lines_to_skip, cell_rect, buf);

            y_cursor += visible_height as u16;
            remaining_height -= visible_height;
            lines_to_skip = 0;

            if remaining_height == 0 {
                break;
            }
        }

        // Render scrollbar with safe bounds
        let safe_content_length = num_lines.max(viewport_height).max(1);
        let safe_scroll_pos =
            scroll_pos.min(safe_content_length.saturating_sub(viewport_height).max(0));

        let mut scroll_state = ScrollbarState::default()
            .content_length(safe_content_length)
            .position(safe_scroll_pos);

        let thumb_style = Style::reset().fg(if state.has_input_focus {
            Color::LightYellow
        } else {
            Color::Gray
        });

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some(SCROLLBAR_UP_SYMBOL))
            .end_symbol(Some(SCROLLBAR_DOWN_SYMBOL))
            .thumb_symbol(SCROLLBAR_THUMB_SYMBOL)
            .track_symbol(Some(SCROLLBAR_TRACK_SYMBOL))
            .thumb_style(thumb_style)
            .track_style(Style::reset().fg(Color::DarkGray));

        scrollbar.render(inner, buf, &mut scroll_state);

        // Update cached values for next render
        state.num_rendered_lines.set(num_lines);
        state.last_viewport_height.set(viewport_height);
    }
}
