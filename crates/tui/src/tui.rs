use crate::utils::mouse_capture::MouseCapture;
use color_eyre::eyre::Result;
use ratatui::{
    crossterm::{
        event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::CrosstermBackend,
    Terminal,
};
use std::io::{stdout, Stdout};

/// Terminal type alias for convenience
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal with proper setup and panic handling
pub fn init() -> Result<(Tui, MouseCapture)> {
    // Install panic hook before any terminal modifications
    set_panic_hook();

    // Enable raw mode for direct terminal control
    enable_raw_mode()?;

    // Initialize mouse capture (enabled by default for trackpad scrolling)
    let mouse_capture = MouseCapture::new_with_capture(true)?;

    // Setup alternate screen and bracketed paste
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;

    // Create and clear terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;

    Ok((terminal, mouse_capture))
}

/// Install a panic hook that restores terminal state before panicking
fn set_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Best effort restoration - we're already panicking
        let _ = restore();
        default_hook(panic_info);
    }));
}

/// Restore the terminal to its original state
///
/// This function should be called before exiting to ensure the terminal
/// is left in a usable state. It disables raw mode, mouse capture,
/// bracketed paste, and returns to the main screen.
pub fn restore() -> Result<()> {
    // The order matters: disable features before leaving alternate screen
    execute!(
        stdout(),
        DisableMouseCapture,
        DisableBracketedPaste,
        LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    Ok(())
}
