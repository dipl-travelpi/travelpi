//! Module `uplink`: the connection to the internet source (the upstream Wi-Fi on the USB
//! adapter), read from NetworkManager and reported as the section `uplink`.
//!
//! The module reads the device right away, again on every state change NetworkManager
//! signals, and at the latest every 2 seconds (signal strength and connectivity check change
//! without a state change). See `README.md` in this folder.

mod networkmanager;
mod state;

use std::time::Duration;

use tokio::time::{Instant, sleep};

use crate::core::reporter::Reporter;
use crate::core::status::{Section, UplinkState, UplinkStatus};
use networkmanager::{NetworkManager, Source};
use state::{DEVICE_FAILED, Device};

/// Interface of the USB Wi-Fi adapter on the development Pi.
pub const INTERFACE: &str = "wlan1";

/// Read the device at least this often, even without a state change.
const REFRESH: Duration = Duration::from_secs(2);
/// Pure changes of the signal strength are reported at most this often (TRA-21, TRA-22).
const SIGNAL_INTERVAL: Duration = Duration::from_secs(5);
/// Pause before trying again when NetworkManager cannot be reached; doubled up to the maximum.
const RETRY_FIRST: Duration = Duration::from_secs(5);
const RETRY_MAX: Duration = Duration::from_secs(60);

/// Error shown when NetworkManager does not answer.
const NOT_REACHABLE: &str = "NetworkManager not reachable";

/// Entry point started by the core.
pub async fn run(reporter: Reporter<UplinkStatus>) {
    run_with(reporter, NetworkManager::default(), INTERFACE).await;
}

async fn run_with(reporter: Reporter<UplinkStatus>, mut source: impl Source, interface: &str) {
    let mut throttle = Throttle::default();
    let mut retry = RETRY_FIRST;
    // Reason of the last failed attempt, kept until a new attempt starts.
    let mut failure: Option<u32> = None;

    loop {
        let device = match source.read(interface).await {
            Ok(device) => {
                retry = RETRY_FIRST;
                if !still_failed(device.as_ref()) {
                    failure = None;
                }
                throttle.report(
                    &reporter,
                    state::status(interface, device.as_ref(), failure),
                );
                device
            }
            Err(error) => {
                tracing::warn!(%error, "cannot read the uplink from NetworkManager");
                reporter.set(UplinkStatus {
                    state: UplinkState::Failed,
                    error: Some(NOT_REACHABLE.into()),
                    ..UplinkStatus::initial()
                });
                sleep(retry).await;
                retry = (retry * 2).min(RETRY_MAX);
                continue;
            }
        };

        let change = source.wait_for_change(REFRESH).await;
        if let (Some(change), Some(last)) = (change, &device)
            && change.state == DEVICE_FAILED
        {
            // NetworkManager is "failed" only for a moment, so report it right away with the
            // reason from the signal, before the next read shows "disconnected" or a retry.
            failure = Some(change.reason);
            let failed = Device {
                state: DEVICE_FAILED,
                ..last.clone()
            };
            throttle.report(&reporter, state::status(interface, Some(&failed), failure));
        }
    }
}

/// A failure stays visible while the device is failed or disconnected, and ends as soon as
/// NetworkManager tries again, connects, or the adapter disappears.
fn still_failed(device: Option<&Device>) -> bool {
    device.is_some_and(|d| {
        matches!(
            state::map_state(d.state),
            UplinkState::Failed | UplinkState::Disconnected
        )
    })
}

/// Limits pure signal changes to one report every [`SIGNAL_INTERVAL`]. Every other change is
/// reported at once, with the current signal.
#[derive(Default)]
struct Throttle {
    last_report: Option<Instant>,
}

impl Throttle {
    fn report(&mut self, reporter: &Reporter<UplinkStatus>, next: UplinkStatus) {
        let current = reporter.current();
        if next == current {
            return;
        }
        let only_signal = UplinkStatus {
            signal: current.signal,
            ..next.clone()
        } == current;
        let too_soon = self
            .last_report
            .is_some_and(|at| at.elapsed() < SIGNAL_INTERVAL);
        if only_signal && too_soon {
            return;
        }
        reporter.set(next);
        self.last_report = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::networkmanager::Change;
    use super::*;
    use crate::core::reporter::channel;
    use std::collections::VecDeque;
    use std::net::Ipv4Addr;
    use std::sync::{Arc, Mutex};
    use tokio::sync::watch;

    /// Stand-in for NetworkManager: answers from a script.
    struct Script {
        reads: VecDeque<Result<Option<Device>, String>>,
        changes: VecDeque<Option<Change>>,
        read_at: Arc<Mutex<Vec<Instant>>>,
    }

    impl Source for Script {
        async fn read(&mut self, _interface: &str) -> Result<Option<Device>, String> {
            self.read_at.lock().unwrap().push(Instant::now());
            // Give the test a chance to see each report.
            tokio::task::yield_now().await;
            match self.reads.pop_front() {
                Some(read) => read,
                None => std::future::pending().await,
            }
        }

        async fn wait_for_change(&mut self, limit: Duration) -> Option<Change> {
            tokio::task::yield_now().await;
            match self.changes.pop_front().flatten() {
                Some(change) => Some(change),
                None => {
                    sleep(limit).await;
                    None
                }
            }
        }
    }

    fn device(state: u32, strength: u8) -> Device {
        Device {
            state,
            ssid: Some("Hotel-Guest".into()),
            strength: Some(strength),
            ipv4: Some(Ipv4Addr::new(192, 0, 2, 15)),
            connectivity: 4,
        }
    }

    /// Starts the module with a script and returns everything it reports, in order.
    async fn reports(
        reads: Vec<Result<Option<Device>, String>>,
        changes: Vec<Option<Change>>,
        duration: Duration,
    ) -> (Vec<UplinkStatus>, Vec<Instant>) {
        let (reporter, mut receiver) = channel::<UplinkStatus>();
        let read_at = Arc::new(Mutex::new(Vec::new()));
        let script = Script {
            reads: reads.into(),
            changes: changes.into(),
            read_at: read_at.clone(),
        };
        let task = tokio::spawn(run_with(reporter, script, "wlan1"));
        let seen = collect(&mut receiver, duration).await;
        task.abort();
        let read_at = read_at.lock().unwrap().clone();
        (seen, read_at)
    }

    async fn collect(
        receiver: &mut watch::Receiver<UplinkStatus>,
        duration: Duration,
    ) -> Vec<UplinkStatus> {
        let mut seen = Vec::new();
        let end = sleep(duration);
        tokio::pin!(end);
        loop {
            tokio::select! {
                changed = receiver.changed() => {
                    if changed.is_err() { break; }
                    seen.push(receiver.borrow_and_update().clone());
                }
                () = &mut end => break,
            }
        }
        seen
    }

    #[tokio::test(start_paused = true)]
    async fn signal_alone_is_reported_at_most_every_5_seconds() {
        // Connected, then a new signal every 2 s: 70, 71, 72, 73, 74.
        let reads = (70..75).map(|s| Ok(Some(device(100, s)))).collect();
        let (seen, _) = reports(reads, vec![], Duration::from_secs(9)).await;
        let signals: Vec<_> = seen.iter().map(|s| s.signal).collect();
        // 70 at 0 s, 71 (2 s) and 72 (4 s) are too soon, 73 at 6 s is reported.
        assert_eq!(signals, [Some(70), Some(73)]);
    }

    #[tokio::test(start_paused = true)]
    async fn other_changes_are_reported_at_once() {
        let reads = vec![Ok(Some(device(100, 70))), Ok(Some(device(30, 71)))];
        let (seen, _) = reports(reads, vec![], Duration::from_secs(3)).await;
        let states: Vec<_> = seen.iter().map(|s| s.state).collect();
        assert_eq!(states, [UplinkState::Connected, UplinkState::Disconnected]);
    }

    #[tokio::test(start_paused = true)]
    async fn failure_from_the_signal_is_shown_until_the_next_attempt() {
        let reads = vec![
            Ok(Some(device(60, 70))), // need-auth: connecting
            Ok(Some(device(30, 70))), // disconnected after the failure
            Ok(Some(device(40, 70))), // prepare: NetworkManager tries again
        ];
        let failed = Some(Change {
            state: DEVICE_FAILED,
            reason: 7,
        });
        let (seen, _) = reports(reads, vec![failed], Duration::from_secs(5)).await;
        let states: Vec<_> = seen.iter().map(|s| (s.state, s.error.clone())).collect();
        let password = Some("wrong or missing password".to_string());
        assert_eq!(
            states,
            [
                (UplinkState::Connecting, None),
                (UplinkState::Failed, password),
                (UplinkState::Connecting, None),
            ]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn missing_networkmanager_is_retried_with_a_growing_pause() {
        let down = || Err("no bus".to_string());
        let reads = vec![down(), down(), down(), Ok(Some(device(30, 0)))];
        let (seen, read_at) = reports(reads, vec![], Duration::from_secs(40)).await;

        assert_eq!(seen[0].state, UplinkState::Failed);
        assert_eq!(seen[0].error.as_deref(), Some(NOT_REACHABLE));
        assert_eq!(seen.last().unwrap().state, UplinkState::Disconnected);

        let pauses: Vec<_> = read_at.windows(2).map(|w| w[1] - w[0]).collect();
        let secs = |s| Duration::from_secs(s);
        assert_eq!(pauses[..3], [secs(5), secs(10), secs(20)]);
    }
}
