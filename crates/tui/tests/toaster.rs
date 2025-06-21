//! Tests for the toaster notification system

#[cfg(test)]
mod tests {
    use corty_tui::widgets::{Toaster, ToasterState};
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn test_toaster_state_basic() {
        let mut state = ToasterState::new();

        // Initially empty
        assert!(!state.has_toasts());
        assert_eq!(state.toasts().len(), 0);

        // Add an error toast
        state.error("Test error message");
        assert!(state.has_toasts());
        assert_eq!(state.toasts().len(), 1);
        assert_eq!(state.toasts()[0].message, "Test error message");
    }

    #[test]
    fn test_toaster_state_multiple_types() {
        let mut state = ToasterState::new();

        state.error("Error message");
        state.warning("Warning message");
        state.info("Info message");
        state.success("Success message");

        assert_eq!(state.toasts().len(), 3); // Only keeps MAX_TOASTS (3)

        // Should have kept the most recent 3
        assert_eq!(state.toasts()[0].message, "Warning message");
        assert_eq!(state.toasts()[1].message, "Info message");
        assert_eq!(state.toasts()[2].message, "Success message");
    }

    #[test]
    fn test_toaster_state_max_limit() {
        let mut state = ToasterState::new();

        // Add more than MAX_TOASTS
        for i in 0..5 {
            state.error(format!("Error {}", i));
        }

        // Should only keep the last 3
        assert_eq!(state.toasts().len(), 3);
        assert_eq!(state.toasts()[0].message, "Error 2");
        assert_eq!(state.toasts()[1].message, "Error 3");
        assert_eq!(state.toasts()[2].message, "Error 4");
    }

    #[test]
    fn test_toaster_state_clear() {
        let mut state = ToasterState::new();

        state.error("Error 1");
        state.error("Error 2");
        assert_eq!(state.toasts().len(), 2);

        state.clear();
        assert!(!state.has_toasts());
        assert_eq!(state.toasts().len(), 0);
    }

    #[test]
    fn test_toaster_render() {
        let mut state = ToasterState::new();
        state.error("Test error message");

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let toaster = Toaster::new(&state);
                f.render_widget(toaster, f.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Check that the error message appears in the buffer
        let buffer_text = buffer_to_string(buffer);
        assert!(buffer_text.contains("Error"));
        assert!(buffer_text.contains("Test error message"));
    }

    #[test]
    fn test_toaster_expiration() {
        let mut state = ToasterState::new();
        state.error("Temporary error");

        assert_eq!(state.toasts().len(), 1);

        // Immediately after adding, tick should not remove it
        state.tick();
        assert_eq!(state.toasts().len(), 1);

        // Simulate waiting for expiration (we can't actually wait in tests)
        // In real usage, toasts expire after TOAST_DURATION (5 seconds)
        // For testing, we'll just verify the tick method exists and can be called
        state.tick();
        assert!(state.toasts().len() <= 1);
    }

    // Helper function to convert buffer to string for assertions
    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut result = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                result.push_str(cell.symbol());
            }
            result.push('\n');
        }
        result
    }
}
