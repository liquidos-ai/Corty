use super::{
    chat_history::{ChatHistory, ChatHistoryState},
    colors,
    command_popup::CommandPopup,
    constants::*,
};
use crate::event::{AppEvent, AppEventSender};
use crate::slash_command::SlashCommand;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Widget},
};
use tui_textarea::{Input, Key, TextArea};

/// State for the chat widget (currently minimal, kept for future expansion)
#[derive(Clone, Debug, Default)]
pub(crate) struct ChatWidgetState {
    // Reserved for future state management
}

/// Main chat widget that handles user input and displays message history
pub(crate) struct ChatWidget<'a> {
    app_event_tx: AppEventSender,
    textarea: TextArea<'a>,
    chat_history_state: ChatHistoryState,
    mouse_capture_active: bool,
    command_popup: Option<CommandPopup>,
    active_command: Option<SlashCommand>,
    fullscreen_mode: bool,
    ai_working_start: Option<std::time::Instant>,
    ai_processing: bool,
    saved_textarea_content: Option<Vec<String>>,
}

impl<'a> ChatWidget<'a> {
    pub(crate) fn new(app_event_tx: AppEventSender) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());

        let chat_history_state = ChatHistoryState::new();

        let mut this = Self {
            app_event_tx,
            textarea,
            chat_history_state,
            mouse_capture_active: true,
            command_popup: None,
            active_command: None,
            fullscreen_mode: false,
            ai_working_start: None,
            ai_processing: false,
            saved_textarea_content: None,
        };
        this.set_placeholder();
        this.set_border();
        this
    }

    /// Set placeholder text for the input area
    fn set_placeholder(&mut self) {
        let placeholder_text = if self.ai_processing {
            PLACEHOLDER_TEXT_AI_PROCESSING
        } else if let Some(cmd) = self.active_command {
            match cmd {
                SlashCommand::AskAI => PLACEHOLDER_TEXT_ASK_AI,
                _ => PLACEHOLDER_TEXT,
            }
        } else {
            PLACEHOLDER_TEXT
        };

        self.textarea.set_placeholder_text(placeholder_text);

        // Use different style when processing
        let placeholder_style = if self.ai_processing {
            Style::default()
                .fg(Color::Rgb(80, 80, 80))
                .add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(colors::DARK_GRAY)
        };
        self.textarea.set_placeholder_style(placeholder_style);

        // Also update cursor style when processing
        if self.ai_processing {
            self.textarea
                .set_cursor_style(Style::default().fg(Color::Rgb(80, 80, 80)));
        } else {
            self.textarea.set_cursor_style(Style::default());
        }
    }

    /// Update the border with current help text and mouse capture status
    fn set_border(&mut self) {
        let mouse_status = if self.mouse_capture_active {
            MOUSE_STATUS_ON
        } else {
            MOUSE_STATUS_OFF
        };

        let fullscreen_indicator = if self.fullscreen_mode {
            FULLSCREEN_INDICATOR
        } else {
            ""
        };

        let title = if self.ai_processing {
            format!("{}{}", fullscreen_indicator, CHAT_TITLE_AI_PROCESSING)
        } else if let Some(cmd) = self.active_command {
            match cmd {
                SlashCommand::AskAI => {
                    format!("{}{}", fullscreen_indicator, CHAT_TITLE_ASK_AI_MODE)
                }
                _ => format!(
                    "{}{} {}",
                    fullscreen_indicator, CHAT_TITLE_NORMAL, mouse_status
                ),
            }
        } else {
            format!(
                "{}{} {}",
                fullscreen_indicator, CHAT_TITLE_NORMAL, mouse_status
            )
        };

        let border_color = if self.ai_processing {
            Color::Rgb(100, 100, 100) // Gray when processing
        } else if self.active_command.is_some() {
            colors::PRIMARY_TEXT
        } else {
            colors::DARK_GRAY
        };

        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_alignment(Alignment::Right),
        );
    }

    /// Handle core AI events
    pub(crate) fn handle_core_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::CoreAI => {
                // Add AI working indicator
                self.add_ai_working();
                self.ai_working_start = Some(std::time::Instant::now());
                self.ai_processing = true;

                // Save current textarea content and clear it
                if !self.textarea.is_empty() {
                    self.saved_textarea_content = Some(self.textarea.lines().to_vec());
                    self.clear_text_area();
                }

                self.set_placeholder();
                self.set_border();
                self.request_redraw();

                // Simulate AI processing with a delay
                let app_event_tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    // Simulate processing delay
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    // Simulate an error for demonstration purposes
                    app_event_tx.send(AppEvent::Error(AI_ERROR_CONNECTION_TIMEOUT.to_string()));

                    // Signal processing complete
                    app_event_tx.send(AppEvent::AiProcessingComplete);
                });
            }
            AppEvent::AiProcessingComplete => {
                // Remove AI working indicator
                self.remove_ai_working();
                self.ai_working_start = None;
                self.ai_processing = false;

                // Restore saved textarea content if any
                if let Some(content) = self.saved_textarea_content.take() {
                    for line in content {
                        self.textarea.insert_str(&line);
                        self.textarea.insert_newline();
                    }
                    // Remove the extra newline at the end
                    if !self.textarea.is_empty() {
                        self.textarea.delete_char();
                    }
                }

                self.set_placeholder();
                self.set_border();

                // Add error response
                self.add_agent_message(AI_ERROR_RESPONSE.to_string());
                self.request_redraw();
            }
            _ => {}
        }
    }

    /// Handle keyboard events, delegating to history or input as appropriate
    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Allow certain shortcuts even when AI is processing
        if self.ai_processing {
            // Only allow escape to cancel (if we add cancellation support later)
            // For now, block all input during processing
            return;
        }

        // If command popup is visible, handle navigation
        if self.command_popup.is_some() {
            self.handle_command_popup_key_event(key_event);
            return;
        }

        // Check if chat history should handle the key event
        let chat_history = ChatHistory::new(self.app_event_tx.clone());

        if chat_history.handle_key_event(key_event, &mut self.chat_history_state) {
            self.request_redraw();
            return;
        }

        // Convert to textarea input and handle
        let input: Input = key_event.into();
        self.handle_textarea_input(input);
    }

    /// Handle keyboard events when command popup is visible
    fn handle_command_popup_key_event(&mut self, key_event: KeyEvent) {
        let input: Input = key_event.into();

        match input {
            // Up/Down: navigate popup
            Input { key: Key::Up, .. } => {
                if let Some(popup) = &mut self.command_popup {
                    popup.select_previous();
                    self.request_redraw();
                }
            }
            Input { key: Key::Down, .. } => {
                if let Some(popup) = &mut self.command_popup {
                    popup.select_next();
                    self.request_redraw();
                }
            }
            // Tab: autocomplete selected command
            Input { key: Key::Tab, .. } => {
                if let Some(popup) = &self.command_popup {
                    if let Some(cmd) = popup.selected_command() {
                        self.textarea.select_all();
                        self.textarea.cut();
                        self.textarea.insert_str(format!("/{} ", cmd.command()));

                        if cmd.requires_input() {
                            // Enter command mode
                            self.active_command = Some(cmd);
                            self.command_popup = None;
                            self.set_placeholder();
                            self.set_border();
                        } else {
                            self.update_command_popup();
                        }
                        self.request_redraw();
                    }
                }
            }
            // Enter: execute selected command or send message
            Input {
                key: Key::Enter,
                shift: false,
                alt: false,
                ctrl: false,
            } => {
                if let Some(popup) = &self.command_popup {
                    if let Some(cmd) = popup.selected_command() {
                        if cmd.requires_input() {
                            // Start command mode - user will type prompt
                            self.active_command = Some(cmd);
                            self.textarea.select_all();
                            self.textarea.cut();
                            self.textarea.insert_str(format!("/{} ", cmd.command()));
                            self.command_popup = None;
                            self.set_placeholder();
                            self.set_border();
                            self.request_redraw();
                            return;
                        } else {
                            // Execute immediately
                            self.execute_slash_command(cmd, None);
                            self.clear_text_area();
                            self.command_popup = None;
                            self.request_redraw();
                            return;
                        }
                    }
                }
                // Fall through to normal input handling
                self.handle_textarea_input(input);
            }
            // Escape: close popup
            Input { key: Key::Esc, .. } => {
                self.command_popup = None;
                self.request_redraw();
            }
            // Other keys: pass to textarea and update popup
            input => {
                self.textarea.input(input);
                self.update_command_popup();
                self.request_redraw();
            }
        }
    }

    /// Handle textarea-specific input events
    fn handle_textarea_input(&mut self, input: Input) {
        match input {
            // Escape: cancel command mode if active
            Input { key: Key::Esc, .. } => {
                if self.active_command.is_some() {
                    self.active_command = None;
                    self.clear_text_area();
                    self.set_placeholder();
                    self.set_border();
                    self.request_redraw();
                }
            }
            // Enter without modifiers: send message
            Input {
                key: Key::Enter,
                shift: false,
                alt: false,
                ctrl: false,
            } => {
                let text = self.textarea.lines().join("\n");

                // Check if we're in command mode
                if let Some(cmd) = self.active_command {
                    // Extract the prompt after the command
                    let prompt = if let Some(stripped) =
                        text.strip_prefix(&format!("{}{} ", COMMAND_PREFIX, cmd.command()))
                    {
                        stripped.to_string()
                    } else {
                        text.clone()
                    };

                    if !prompt.trim().is_empty() {
                        self.execute_slash_command(cmd, Some(prompt));
                    }

                    self.clear_text_area();
                    self.active_command = None;
                    self.set_placeholder();
                    self.set_border();
                } else if !text.trim().is_empty() {
                    // Default behavior: send message to AI (same as /ask-ai)
                    self.add_user_message(text.clone());
                    self.clear_text_area();
                    self.dispatch_event(AppEvent::Core);
                }
            }
            // Ctrl+N: insert newline
            Input {
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => {
                self.textarea.insert_newline();
            }
            // All other inputs: pass through to textarea
            input => {
                self.textarea.input(input);
                self.update_command_popup();
            }
        }
        self.request_redraw();
    }

    /// Clear the input textarea
    fn clear_text_area(&mut self) {
        self.textarea.select_all();
        self.textarea.cut();
        self.set_placeholder();
    }

    /// Add a user message to the chat history
    fn add_user_message(&mut self, message: String) {
        self.chat_history_state.add_user_message(message);
    }

    /// Add an agent message to the chat history
    fn add_agent_message(&mut self, message: String) {
        self.chat_history_state.add_agent_message(message);
    }

    /// Add AI working indicator to the chat history
    fn add_ai_working(&mut self) {
        self.chat_history_state.add_ai_working();
    }

    /// Remove AI working indicator from the chat history
    fn remove_ai_working(&mut self) {
        self.chat_history_state.remove_ai_working();
    }

    /// Send an event through the application event system
    fn dispatch_event(&mut self, event: AppEvent) {
        self.app_event_tx.send(event);
    }

    /// Update mouse capture state and refresh UI elements
    pub(crate) fn set_mouse_capture_active(&mut self, is_active: bool) {
        self.mouse_capture_active = is_active;
        self.set_border();
        self.chat_history_state.set_mouse_capture_active(is_active);
    }

    /// Update fullscreen mode state
    pub(crate) fn set_fullscreen_mode(&mut self, is_fullscreen: bool) {
        self.fullscreen_mode = is_fullscreen;
        self.set_border();
    }

    /// Handle scroll events with optional magnification for faster scrolling
    pub(crate) fn handle_scroll_event(&mut self, scroll_delta: i32) {
        // Single-line scrolls are preserved, multi-line scrolls are magnified
        let magnified_delta = if scroll_delta.abs() == 1 {
            scroll_delta
        } else {
            scroll_delta * 2
        };

        let chat_history = ChatHistory::new(self.app_event_tx.clone());
        chat_history.scroll(magnified_delta, &mut self.chat_history_state);
        self.request_redraw();
    }

    /// Request a UI redraw
    fn request_redraw(&self) {
        self.app_event_tx.send(AppEvent::Redraw);
    }

    /// Update command popup based on current textarea content
    fn update_command_popup(&mut self) {
        // Don't show popup if we're already in command mode
        if self.active_command.is_some() {
            return;
        }

        let text = self.textarea.lines().join("\n");

        // Check if we should show the popup
        if text.starts_with(COMMAND_PREFIX) && !text.contains(' ') {
            // Create or update popup
            if self.command_popup.is_none() {
                self.command_popup = Some(CommandPopup::new());
            }

            if let Some(popup) = &mut self.command_popup {
                popup.update_filter(&text);

                // Hide popup if no commands match
                if !popup.has_commands() {
                    self.command_popup = None;
                }
            }
        } else {
            // Hide popup if not starting with '/' or has space
            self.command_popup = None;
        }
    }

    /// Execute a slash command with optional prompt
    fn execute_slash_command(&mut self, command: SlashCommand, prompt: Option<String>) {
        match command {
            SlashCommand::Clear => {
                self.chat_history_state.clear();
                self.request_redraw();
            }
            SlashCommand::AskAI => {
                if let Some(prompt) = prompt {
                    // Send the user's prompt to the AI
                    self.add_user_message(prompt.clone());
                    self.dispatch_event(AppEvent::Core);
                }
            }
            SlashCommand::Fullscreen => {
                self.dispatch_event(AppEvent::ToggleFullscreen);
                self.fullscreen_mode = !self.fullscreen_mode;
                self.set_border();
                self.request_redraw();
            }
        }
    }

    /// Update AI working animation and return true if redraw is needed
    pub(crate) fn update_ai_working_animation(&mut self) -> bool {
        if let Some(start_time) = self.ai_working_start {
            let elapsed = start_time.elapsed().as_millis() as u64;
            self.chat_history_state.update_ai_working(elapsed);
            true
        } else {
            false
        }
    }

    /// Render the chat widget with history and input areas
    fn render_view(&mut self, area: Rect, buf: &mut Buffer) {
        // Calculate textarea height (lines + border)
        let textarea_height = (self.textarea.lines().len() as u16 + 2).max(3);

        // Calculate command popup height if visible
        let popup_height = self
            .command_popup
            .as_ref()
            .map(|p| p.calculate_height())
            .unwrap_or(0);

        // Split into three areas: input, popup (if visible), and history
        let constraints = if popup_height > 0 {
            vec![
                Constraint::Length(textarea_height), // Input area at top
                Constraint::Length(popup_height),    // Command popup below input
                Constraint::Min(0),                  // History area takes remaining space
            ]
        } else {
            vec![
                Constraint::Length(textarea_height), // Input area at top
                Constraint::Min(0),                  // History area takes remaining space
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Update placeholder if needed
        if self
            .textarea
            .lines()
            .iter()
            .all(|line| line.trim().is_empty())
        {
            self.set_placeholder();
        }

        // Render input textarea at the top (always first chunk)
        self.textarea.render(chunks[0], buf);

        // Render command popup if visible
        if let Some(popup) = &self.command_popup {
            popup.render(chunks[1], buf);
            // History area is in chunks[2]
            let chat_history = ChatHistory::new(self.app_event_tx.clone());
            chat_history.render(chunks[2], buf, &mut self.chat_history_state);
        } else {
            // History area is in chunks[1]
            let chat_history = ChatHistory::new(self.app_event_tx.clone());
            chat_history.render(chunks[1], buf, &mut self.chat_history_state);
        }
    }
}

impl<'a> Widget for &mut ChatWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_view(area, buf);
    }
}
