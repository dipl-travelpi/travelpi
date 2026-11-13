//! How a module reports its section to the core.
//!
//! A module is an async task. It gets a [`Reporter`] for its section and calls
//! [`Reporter::set`] with the whole section whenever something changes. The core keeps the
//! latest value for `GET /v0/status` and sends an event to every client of `/v0/events`.
//!
//! ```ignore
//! pub async fn run(reporter: Reporter<ApStatus>) {
//!     loop {
//!         let status = read_state_of_access_point().await;
//!         reporter.set(status);
//!         // wait for the next change ...
//!     }
//! }
//! ```

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::{Instant, sleep};

use crate::core::status::Section;

/// First pause before a module that ended is started again.
const RESTART_FIRST: Duration = Duration::from_secs(5);
/// Longest pause; the pause doubles after every restart up to this value.
const RESTART_MAX: Duration = Duration::from_secs(60);

/// Handle a module uses to report its section.
#[derive(Clone)]
pub struct Reporter<T: Section> {
    sender: Arc<watch::Sender<T>>,
}

impl<T: Section> Reporter<T> {
    /// Replaces the section. Nothing is sent if the value is the same as before.
    pub fn set(&self, value: T) {
        self.sender.send_if_modified(|current| {
            if *current == value {
                false
            } else {
                *current = value;
                true
            }
        });
    }

    /// The value reported last.
    pub fn current(&self) -> T {
        self.sender.borrow().clone()
    }
}

/// Creates the channel for one section, starting with its initial state.
/// The core keeps the receiver; the module gets the reporter.
pub fn channel<T: Section>() -> (Reporter<T>, watch::Receiver<T>) {
    let (sender, receiver) = watch::channel(T::initial());
    let reporter = Reporter {
        sender: Arc::new(sender),
    };
    (reporter, receiver)
}

/// Starts a module task and keeps it running.
///
/// If the task ends or panics, the core logs it, marks the section as stopped (so that
/// clients do not see an outdated state) and starts the module again after a pause: 5 s,
/// then doubled up to 60 s. A module that ran for longer than the longest pause starts
/// again with the first pause.
pub fn spawn_module<T, F, Fut>(reporter: Reporter<T>, run: F)
where
    T: Section,
    F: Fn(Reporter<T>) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        let mut pause = RESTART_FIRST;
        loop {
            let started = Instant::now();
            match tokio::spawn(run(reporter.clone())).await {
                Ok(()) => tracing::error!(section = T::NAME, "module task ended"),
                Err(error) => tracing::error!(section = T::NAME, %error, "module task panicked"),
            }
            reporter.set(reporter.current().stopped());

            if started.elapsed() > RESTART_MAX {
                pause = RESTART_FIRST;
            }
            tracing::info!(
                section = T::NAME,
                seconds = pause.as_secs(),
                "restarting module"
            );
            sleep(pause).await;
            pause = (pause * 2).min(RESTART_MAX);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::status::{ApState, ApStatus, MODULE_STOPPED};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn running() -> ApStatus {
        ApStatus {
            state: ApState::Running,
            ..ApStatus::initial()
        }
    }

    #[tokio::test]
    async fn same_value_is_not_sent_again() {
        let (reporter, mut receiver) = channel::<ApStatus>();
        receiver.mark_unchanged();

        reporter.set(ApStatus::initial());
        assert!(!receiver.has_changed().unwrap());

        reporter.set(running());
        assert!(receiver.has_changed().unwrap());
        assert_eq!(receiver.borrow_and_update().state, ApState::Running);
    }

    #[tokio::test(start_paused = true)]
    async fn ended_module_is_marked_stopped_and_restarted() {
        let starts = Arc::new(AtomicUsize::new(0));
        let (reporter, mut receiver) = channel::<ApStatus>();
        let counter = starts.clone();
        spawn_module(reporter, move |reporter| {
            counter.fetch_add(1, Ordering::SeqCst);
            async move { reporter.set(running()) }
        });

        let status = receiver
            .wait_for(|s| s.error.is_some())
            .await
            .unwrap()
            .clone();
        assert_eq!(status.error.as_deref(), Some(MODULE_STOPPED));
        assert_eq!(status.state, ApState::Failed);
        assert_eq!(starts.load(Ordering::SeqCst), 1);

        // Not before the first pause ...
        sleep(RESTART_FIRST - Duration::from_millis(100)).await;
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        // ... but right after it.
        sleep(Duration::from_millis(200)).await;
        assert_eq!(starts.load(Ordering::SeqCst), 2);
        // The second pause is twice as long.
        sleep(RESTART_FIRST * 2 - Duration::from_millis(200)).await;
        assert_eq!(starts.load(Ordering::SeqCst), 2);
        sleep(Duration::from_millis(400)).await;
        assert_eq!(starts.load(Ordering::SeqCst), 3);

        // The channel stays open, or every event stream would end.
        assert!(receiver.has_changed().is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn panicking_module_is_restarted() {
        let starts = Arc::new(AtomicUsize::new(0));
        let (reporter, _receiver) = channel::<ApStatus>();
        let counter = starts.clone();
        spawn_module(reporter, move |_reporter| {
            counter.fetch_add(1, Ordering::SeqCst);
            async { panic!("test panic") }
        });

        sleep(RESTART_FIRST + Duration::from_millis(100)).await;
        assert_eq!(starts.load(Ordering::SeqCst), 2);
    }
}
