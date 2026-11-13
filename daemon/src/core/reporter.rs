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

use tokio::sync::watch;

use crate::core::status::Section;

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

/// Starts a module task. If the task ends or panics, the core logs it and marks the
/// section as stopped, so that clients see it instead of an outdated state.
pub fn spawn_module<T, F, Fut>(reporter: Reporter<T>, run: F)
where
    T: Section,
    F: FnOnce(Reporter<T>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let task = tokio::spawn(run(reporter.clone()));
    tokio::spawn(async move {
        match task.await {
            Ok(()) => tracing::error!(section = T::NAME, "module task ended"),
            Err(error) => tracing::error!(section = T::NAME, %error, "module task panicked"),
        }
        reporter.set(reporter.current().stopped());
        // Keep the reporter: if the last one were dropped, the event stream would end.
        std::future::pending::<()>().await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::status::{ApStatus, MODULE_STOPPED};

    #[tokio::test]
    async fn same_value_is_not_sent_again() {
        let (reporter, mut receiver) = channel::<ApStatus>();
        receiver.mark_unchanged();

        reporter.set(ApStatus::initial());
        assert!(!receiver.has_changed().unwrap());

        reporter.set(ApStatus {
            running: true,
            ..ApStatus::initial()
        });
        assert!(receiver.has_changed().unwrap());
        assert!(receiver.borrow_and_update().running);
    }

    #[tokio::test]
    async fn ended_module_is_marked_stopped() {
        let (reporter, mut receiver) = channel::<ApStatus>();
        spawn_module(reporter, |reporter| async move {
            reporter.set(ApStatus {
                running: true,
                ..ApStatus::initial()
            });
        });

        let status = receiver
            .wait_for(|s| s.error.is_some())
            .await
            .unwrap()
            .clone();
        assert_eq!(status.error.as_deref(), Some(MODULE_STOPPED));
        assert!(!status.running);

        // The channel must stay open, or every event stream would end.
        tokio::task::yield_now().await;
        assert!(receiver.has_changed().is_ok());
    }
}
