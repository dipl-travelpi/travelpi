//! Simulation mode (`--simulate`): made-up values for every section, so that the dashboard
//! and the API can be used on a laptop without the real hardware.
//!
//! These two tasks are also the example of how a module looks: an async function that gets
//! a `Reporter` for its section and reports the whole section whenever something changes.

use std::net::Ipv4Addr;
use std::time::Duration;

use tokio::time::sleep;

use crate::core::reporter::Reporter;
use crate::core::status::{ApStatus, UplinkState, UplinkStatus};

/// Simulated own Wi-Fi: running, with a number of devices that changes every 10 s.
pub async fn ap(reporter: Reporter<ApStatus>) {
    let running = ApStatus {
        interface: Some("wlan0".into()),
        running: true,
        ssid: Some("TravelPi".into()),
        channel: Some(6),
        subnet: Some("10.157.1.0/24".into()),
        clients: Some(0),
        error: None,
    };
    for clients in [0, 1, 2, 3, 2, 1].into_iter().cycle() {
        reporter.set(ApStatus {
            clients: Some(clients),
            ..running.clone()
        });
        sleep(Duration::from_secs(10)).await;
    }
}

/// Simulated internet source: connects, stays connected for a while with a changing
/// signal, loses the network and connects again.
pub async fn uplink(reporter: Reporter<UplinkStatus>) {
    let base = UplinkStatus {
        interface: Some("wlan1".into()),
        state: UplinkState::Disconnected,
        ssid: None,
        signal: None,
        ipv4: None,
        internet: None,
        error: None,
    };
    let ssid = Some("Hotel-Guest".to_string());
    let address = Some(Ipv4Addr::new(192, 0, 2, 15));

    loop {
        reporter.set(base.clone());
        sleep(Duration::from_secs(2)).await;

        reporter.set(UplinkStatus {
            state: UplinkState::Connecting,
            ssid: ssid.clone(),
            signal: Some(70),
            ..base.clone()
        });
        sleep(Duration::from_secs(3)).await;

        let connected = UplinkStatus {
            state: UplinkState::Connected,
            ssid: ssid.clone(),
            signal: Some(70),
            ipv4: address,
            ..base.clone()
        };
        reporter.set(connected.clone());
        sleep(Duration::from_secs(1)).await;

        // Signal changes are reported at most every 5 s, as agreed for `uplink`.
        for signal in [72, 74, 71, 68, 66, 69, 73, 75, 72, 70, 67, 71] {
            reporter.set(UplinkStatus {
                signal: Some(signal),
                internet: Some(true),
                ..connected.clone()
            });
            sleep(Duration::from_secs(5)).await;
        }

        reporter.set(UplinkStatus {
            state: UplinkState::Failed,
            error: Some("network not found".into()),
            internet: Some(false),
            ..base.clone()
        });
        sleep(Duration::from_secs(5)).await;
    }
}
