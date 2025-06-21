use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use std::io::{stdout, Result};

/// Manages the mouse capture state for the terminal
///
/// This struct ensures that mouse events are properly enabled/disabled
/// and cleaned up on drop to prevent terminal state corruption.
pub(crate) struct MouseCapture {
    is_active: bool,
}

impl MouseCapture {
    /// Creates a new MouseCapture instance with the specified initial state
    pub(crate) fn new_with_capture(initial_state: bool) -> Result<Self> {
        let mut capture = Self { is_active: false };

        if initial_state {
            capture.enable()?;
        }

        Ok(capture)
    }

    /// Enable mouse capture
    fn enable(&mut self) -> Result<()> {
        if !self.is_active {
            enable_capture()?;
            self.is_active = true;
        }
        Ok(())
    }
}

impl MouseCapture {
    /// Check if mouse capture is currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Set the mouse capture state (idempotent)
    pub fn set_active(&mut self, is_active: bool) -> Result<()> {
        match (self.is_active, is_active) {
            (true, true) | (false, false) => Ok(()),
            (true, false) => self.disable(),
            (false, true) => self.enable(),
        }
    }

    /// Toggle the mouse capture state
    pub(crate) fn toggle(&mut self) -> Result<()> {
        self.set_active(!self.is_active)
    }

    /// Disable mouse capture
    pub(crate) fn disable(&mut self) -> Result<()> {
        if self.is_active {
            disable_capture()?;
            self.is_active = false;
        }
        Ok(())
    }
}

impl Drop for MouseCapture {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors during shutdown
        let _ = self.disable();
    }
}

/// Enable mouse capture in the terminal
fn enable_capture() -> Result<()> {
    execute!(stdout(), EnableMouseCapture)
}

/// Disable mouse capture in the terminal
fn disable_capture() -> Result<()> {
    execute!(stdout(), DisableMouseCapture)
}
