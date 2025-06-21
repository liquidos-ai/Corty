use ratatui::crossterm::event::KeyEvent;
use std::sync::mpsc::{channel as std_channel, Sender as StdSender};
use tokio::sync::mpsc::Sender;

/// Application-wide events that drive the UI state machine
#[derive(Debug)]
pub(crate) enum AppEvent {
    /// Request to redraw the UI
    Redraw,

    /// Core processing event
    Core,

    /// AI core processing event
    CoreAI,

    /// Scroll event with delta (positive = down, negative = up)
    Scroll(i32),

    /// Keyboard input event
    Key(KeyEvent),

    /// Request to exit the application gracefully
    ExitRequest,

    /// Mouse capture state has changed
    MouseCaptureChanged(bool),

    /// Toggle fullscreen mode (hide/show welcome widget)
    ToggleFullscreen,

    /// Error event with message
    Error(String),

    /// AI processing has completed
    AiProcessingComplete,
}

/// Thread-safe event sender that can be used from both async and sync contexts
#[derive(Clone)]
pub struct AppEventSender {
    tx: Sender<AppEvent>,
    blocking_tx: StdSender<AppEvent>,
}

impl AppEventSender {
    /// Creates a new event sender with both async and sync channels
    pub fn new(tx: Sender<AppEvent>) -> Self {
        let (blocking_tx, blocking_rx) = std_channel();

        // Spawn a task to forward events from the blocking channel to the async channel
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Ok(event) = blocking_rx.recv() {
                if tx_clone.send(event).await.is_err() {
                    // Channel closed, exit the forwarding task
                    break;
                }
            }
        });

        Self { tx, blocking_tx }
    }

    /// Send an event through the appropriate channel based on the current context
    pub fn send(&self, event: AppEvent) {
        // Try to detect if we're in a tokio runtime context
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let tx = self.tx.clone();
            // We're in async context, use tokio::spawn
            handle.spawn(async move {
                let _ = tx.send(event).await;
            });
        } else {
            // We're in sync context, use the blocking channel
            let _ = self.blocking_tx.send(event);
        }
    }
}
