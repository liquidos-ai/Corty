use crate::event::{AppEvent, AppEventSender};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub(crate) struct ScrollEventHelper {
    app_event_tx: AppEventSender,
    scroll_delta: Arc<AtomicI32>,
    timer_scheduled: Arc<AtomicBool>,
}

/// How long to wait after the first scroll event before sending the
/// accumulated scroll delta to the main thread.
const DEBOUNCE_WINDOW: Duration = Duration::from_millis(100);

/// Utility to debounce scroll events so we can determine the magnitude of
/// each scroll burst by accumulating individual wheel events over a short
/// window. Uses a separate thread for timing to avoid requiring a Tokio runtime.
impl ScrollEventHelper {
    pub(crate) fn new(app_event_tx: AppEventSender) -> Self {
        Self {
            app_event_tx,
            scroll_delta: Arc::new(AtomicI32::new(0)),
            timer_scheduled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn scroll_up(&self) {
        self.scroll_delta.fetch_sub(1, Ordering::Relaxed);
        self.schedule_notification();
    }

    pub(crate) fn scroll_down(&self) {
        self.scroll_delta.fetch_add(1, Ordering::Relaxed);
        self.schedule_notification();
    }

    /// Starts a one-shot timer **only once** per burst of wheel events.
    fn schedule_notification(&self) {
        // If the timer is already scheduled, do nothing.
        if self
            .timer_scheduled
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        // Otherwise, schedule a new timer.
        let tx = self.app_event_tx.clone();
        let delta = Arc::clone(&self.scroll_delta);
        let timer_flag = Arc::clone(&self.timer_scheduled);

        // Schedule the timer on a separate thread to avoid requiring a Tokio runtime
        thread::spawn(move || {
            thread::sleep(DEBOUNCE_WINDOW);

            let accumulated = delta.swap(0, Ordering::SeqCst);
            if accumulated != 0 {
                tx.send(AppEvent::Scroll(accumulated));
            }

            timer_flag.store(false, Ordering::SeqCst);
        });
    }
}
