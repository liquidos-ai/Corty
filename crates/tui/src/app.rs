use crate::{
    event::{AppEvent, AppEventSender},
    utils::{mouse_capture::MouseCapture, scroll_event_helper::ScrollEventHelper},
    widgets::{
        constants::{self, ERROR_TOGGLE_MOUSE_MODE},
        ChatWidget, ChatWidgetState, Toaster, ToasterState, WelcomeWidget,
    },
};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Margin},
    Frame,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::{interval, Duration as TokioDuration};
use tokio::{
    sync::mpsc::{channel, Receiver},
    task::JoinHandle,
};

pub(crate) enum AppState<'a> {
    Chat {
        widget: ChatWidget<'a>,
        #[allow(dead_code)]
        state: ChatWidgetState,
    },
}

pub(crate) struct App<'a> {
    app_event_rx: Receiver<AppEvent>,
    app_event_tx: AppEventSender,
    app_state: AppState<'a>,
    blocking_task: Option<JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
    fullscreen_mode: bool,
    toaster_state: ToasterState,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        let app_event_tx = AppEventSender::new(tx);

        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag_clone = shutdown_flag.clone();

        let task = {
            let app_event_tx = app_event_tx.clone();
            let scroll_event_helper = ScrollEventHelper::new(app_event_tx.clone());
            Some(tokio::task::spawn_blocking(move || {
                use ratatui::crossterm::event::{poll, read};
                while !shutdown_flag_clone.load(Ordering::SeqCst) {
                    if poll(Duration::from_millis(100)).unwrap_or(false) {
                        if let Ok(event) = read() {
                            match event {
                                Event::Key(key_event) => {
                                    app_event_tx.send(AppEvent::Key(key_event));
                                }
                                Event::Resize(_, _) => {
                                    app_event_tx.send(AppEvent::Redraw);
                                }
                                Event::Mouse(MouseEvent {
                                    kind: MouseEventKind::ScrollUp,
                                    ..
                                }) => {
                                    scroll_event_helper.scroll_up();
                                }
                                Event::Mouse(MouseEvent {
                                    kind: MouseEventKind::ScrollDown,
                                    ..
                                }) => {
                                    scroll_event_helper.scroll_down();
                                }
                                Event::Paste(pasted) => {
                                    use ratatui::crossterm::event::KeyModifiers;

                                    for ch in pasted.chars() {
                                        let key_event = match ch {
                                            '\n' | '\r' => {
                                                // Represent newline as <Shift+Enter> so that the chat
                                                // widget treats it as a literal newline instead of a submit
                                                // action (submission is only triggered on Enter *without*
                                                // any modifiers).
                                                KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)
                                            }
                                            _ => KeyEvent::new(
                                                KeyCode::Char(ch),
                                                KeyModifiers::empty(),
                                            ),
                                        };
                                        app_event_tx.send(AppEvent::Key(key_event));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }))
        };

        let chat_state = ChatWidgetState::default();
        let mut chat_widget = ChatWidget::new(app_event_tx.clone());
        // Set initial mouse capture state
        chat_widget.set_mouse_capture_active(true);
        let app_state = AppState::Chat {
            widget: chat_widget,
            state: chat_state,
        };

        Self {
            app_event_tx,
            app_event_rx: rx,
            app_state,
            blocking_task: task,
            shutdown_flag,
            fullscreen_mode: false,
            toaster_state: ToasterState::new(),
        }
    }

    fn render(&mut self, frame: &mut Frame<'_>) {
        let padded_area = frame.area().inner(Margin {
            vertical: constants::APP_PADDING,
            horizontal: constants::APP_PADDING,
        });

        if self.fullscreen_mode {
            // In fullscreen mode, use the entire padded area for the chat widget
            match &mut self.app_state {
                AppState::Chat { widget, state: _ } => {
                    frame.render_widget(widget, padded_area);
                }
            }
        } else {
            // Normal mode with welcome widget
            let welcome_widget = WelcomeWidget::new();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(constants::WELCOME_WIDGET_HEIGHT),
                    Constraint::Min(0),
                ])
                .split(padded_area);
            frame.render_widget(welcome_widget, layout[0]);

            match &mut self.app_state {
                AppState::Chat { widget, state: _ } => {
                    frame.render_widget(widget, layout[1]);
                }
            }
        }

        // Always render toaster on top of everything else
        let toaster = Toaster::new(&self.toaster_state);
        frame.render_widget(toaster, frame.area());
    }

    fn dispatch_key_event(&mut self, key_event: KeyEvent) {
        match &mut self.app_state {
            AppState::Chat { widget, .. } => {
                widget.handle_key_event(key_event);
            }
        }
    }

    fn dispatch_scroll_event(&mut self, scroll_delta: i32) {
        match &mut self.app_state {
            AppState::Chat { widget, .. } => {
                widget.handle_scroll_event(scroll_delta);
            }
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut super::tui::Tui,
        mouse_capture: &mut MouseCapture,
    ) -> color_eyre::Result<()> {
        self.app_event_tx.send(AppEvent::Redraw);

        // Create a ticker for toaster updates
        let mut ticker = interval(TokioDuration::from_millis(100));

        loop {
            tokio::select! {
                Some(event) = self.app_event_rx.recv() => {
                    match event {
                        AppEvent::Redraw => {
                            terminal.draw(|f| self.render(f))?;
                        }
                        AppEvent::Key(event) => {
                            // Handle Ctrl+Y or Ctrl+T before dispatching to widgets
                            if matches!(
                                event,
                                KeyEvent {
                                    code: KeyCode::Char('y') | KeyCode::Char('Y'),
                                    modifiers: KeyModifiers::CONTROL,
                                    ..
                                }
                            ) {
                                if mouse_capture.toggle().is_err() {
                                    self.toaster_state.error(ERROR_TOGGLE_MOUSE_MODE);
                                    self.app_event_tx.send(AppEvent::Redraw);
                                } else {
                                    let is_active = mouse_capture.is_active();
                                    self.app_event_tx
                                        .send(AppEvent::MouseCaptureChanged(is_active));
                                }
                            } else if let KeyEvent {
                                code: KeyCode::Char('d'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } = event
                            {
                                self.app_event_tx.send(AppEvent::ExitRequest);
                            } else {
                                self.dispatch_key_event(event);
                            }
                        }
                        AppEvent::Scroll(scroll_delta) => {
                            self.dispatch_scroll_event(scroll_delta);
                        }
                        AppEvent::Core => {
                            self.app_event_tx.send(AppEvent::CoreAI);
                        }
                        AppEvent::CoreAI => match &mut self.app_state {
                            AppState::Chat { widget, .. } => {
                                widget.handle_core_event(AppEvent::CoreAI);
                            }
                        },
                        AppEvent::AiProcessingComplete => match &mut self.app_state {
                            AppState::Chat { widget, .. } => {
                                widget.handle_core_event(AppEvent::AiProcessingComplete);
                            }
                        },
                        AppEvent::ExitRequest => break,
                        AppEvent::MouseCaptureChanged(is_active) => {
                            match &mut self.app_state {
                                AppState::Chat { widget, .. } => {
                                    widget.set_mouse_capture_active(is_active);
                                }
                            }
                            self.app_event_tx.send(AppEvent::Redraw);
                        }
                        AppEvent::ToggleFullscreen => {
                            self.fullscreen_mode = !self.fullscreen_mode;
                            // Update the chat widget's fullscreen state
                            match &mut self.app_state {
                                AppState::Chat { widget, .. } => {
                                    widget.set_fullscreen_mode(self.fullscreen_mode);
                                }
                            }
                            self.app_event_tx.send(AppEvent::Redraw);
                        }
                        AppEvent::Error(message) => {
                            self.toaster_state.error(message);
                            self.app_event_tx.send(AppEvent::Redraw);
                        }
                    }
                }
                _ = ticker.tick() => {
                    // Update toaster state to remove expired toasts
                    self.toaster_state.tick();

                    // Update AI working animation
                    let needs_redraw = match &mut self.app_state {
                        AppState::Chat { widget, .. } => {
                            widget.update_ai_working_animation()
                        }
                    };

                    if self.toaster_state.has_toasts() || needs_redraw {
                        self.app_event_tx.send(AppEvent::Redraw);
                    }
                }
            }
        }

        super::tui::restore()?;
        self.shutdown_flag.store(true, Ordering::SeqCst);
        if let Some(task) = self.blocking_task.take() {
            let _ = task.await;
        }
        Ok(())
    }
}
