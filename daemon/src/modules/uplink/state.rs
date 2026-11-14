//! Turns what NetworkManager says about the uplink device into the section `uplink`.
//!
//! Everything here is a plain function without D-Bus, so it can be tested on any computer.

use std::net::Ipv4Addr;

use crate::core::status::{Section, UplinkState, UplinkStatus};

/// What the module reads from NetworkManager about one device.
#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    /// Device state (`NMDeviceState`), for example 100 for "activated".
    pub state: u32,
    /// Network name of the access point the device is connected or connecting to.
    pub ssid: Option<String>,
    /// Signal strength of that access point in percent.
    pub strength: Option<u8>,
    /// First IPv4 address of the device.
    pub ipv4: Option<Ipv4Addr>,
    /// Result of the IPv4 connectivity check (`NMConnectivityState`).
    pub connectivity: u32,
}

// Device states of NetworkManager (`NMDeviceState`), only those that need a name here.
// https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMDeviceState
const DEVICE_DISCONNECTED: u32 = 30;
const DEVICE_PREPARE: u32 = 40;
const DEVICE_SECONDARIES: u32 = 90;
const DEVICE_ACTIVATED: u32 = 100;
const DEVICE_DEACTIVATING: u32 = 110;
pub const DEVICE_FAILED: u32 = 120;

// Results of the connectivity check (`NMConnectivityState`).
const CONNECTIVITY_UNKNOWN: u32 = 0;
const CONNECTIVITY_FULL: u32 = 4;

/// Maps the device state of NetworkManager onto the five states of the API.
///
/// NetworkManager has thirteen device states; the API only needs five:
///
/// | NetworkManager | API |
/// |---|---|
/// | unknown, unmanaged, unavailable (0 to 20) | `unavailable` |
/// | disconnected, deactivating (30, 110) | `disconnected` |
/// | prepare to secondaries (40 to 90): config, need-auth, ip-config, ... | `connecting` |
/// | activated (100) | `connected` |
/// | failed (120) | `failed` |
pub fn map_state(state: u32) -> UplinkState {
    match state {
        DEVICE_ACTIVATED => UplinkState::Connected,
        DEVICE_FAILED => UplinkState::Failed,
        DEVICE_PREPARE..=DEVICE_SECONDARIES => UplinkState::Connecting,
        DEVICE_DISCONNECTED | DEVICE_DEACTIVATING => UplinkState::Disconnected,
        _ => UplinkState::Unavailable,
    }
}

/// Short English text for the reason of a failed connection (`NMDeviceStateReason`).
///
/// Only our own texts go into the API, never raw output of NetworkManager.
/// https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMDeviceStateReason
pub fn failure_text(reason: u32) -> &'static str {
    match reason {
        // no-secrets: NetworkManager asked for a (new) password and got none, which is what
        // happens after a wrong password
        7 => "wrong or missing password",
        // supplicant-disconnect, -config-failed, -failed, -timeout
        8..=11 => "authentication failed",
        // ip-config-unavailable, dhcp-start-failed, dhcp-error, dhcp-failed
        5 | 17..=19 => "no address received",
        // ssid-not-found
        53 => "network not found",
        _ => "connection failed",
    }
}

/// Builds the section `uplink`.
///
/// `device` is `None` if NetworkManager does not know the interface (adapter missing).
/// `failure` is the reason of the last failed attempt, as long as no new attempt has started;
/// NetworkManager itself shows "failed" only for a moment and then "disconnected".
pub fn status(interface: &str, device: Option<&Device>, failure: Option<u32>) -> UplinkStatus {
    let Some(device) = device else {
        return UplinkStatus::initial();
    };

    let mut state = map_state(device.state);
    if state == UplinkState::Disconnected && failure.is_some() {
        state = UplinkState::Failed;
    }

    let on_network = matches!(state, UplinkState::Connecting | UplinkState::Connected);
    let connected = state == UplinkState::Connected;

    UplinkStatus {
        interface: Some(interface.to_string()),
        state,
        ssid: device.ssid.clone().filter(|_| on_network),
        signal: device.strength.filter(|_| on_network).map(|s| s.min(100)),
        ipv4: device.ipv4.filter(|_| connected),
        internet: if connected {
            internet(device.connectivity)
        } else {
            None
        },
        error: (state == UplinkState::Failed).then(|| failure_text(failure.unwrap_or(0)).into()),
    }
}

/// `true` only with full internet access; `false` with none, limited or a captive portal;
/// `None` while the check has not run.
fn internet(connectivity: u32) -> Option<bool> {
    match connectivity {
        CONNECTIVITY_UNKNOWN => None,
        CONNECTIVITY_FULL => Some(true),
        _ => Some(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(state: u32) -> Device {
        Device {
            state,
            ssid: Some("Hotel-Guest".into()),
            strength: Some(72),
            ipv4: Some(Ipv4Addr::new(192, 0, 2, 15)),
            connectivity: CONNECTIVITY_FULL,
        }
    }

    #[test]
    fn every_device_state_has_an_api_state() {
        let cases = [
            (0, UplinkState::Unavailable),
            (10, UplinkState::Unavailable),
            (20, UplinkState::Unavailable),
            (30, UplinkState::Disconnected),
            (40, UplinkState::Connecting),
            (50, UplinkState::Connecting),
            (60, UplinkState::Connecting),
            (70, UplinkState::Connecting),
            (80, UplinkState::Connecting),
            (90, UplinkState::Connecting),
            (100, UplinkState::Connected),
            (110, UplinkState::Disconnected),
            (120, UplinkState::Failed),
        ];
        for (nm, api) in cases {
            assert_eq!(map_state(nm), api, "device state {nm}");
        }
    }

    #[test]
    fn missing_adapter_is_unavailable_without_interface() {
        assert_eq!(status("wlan1", None, None), UplinkStatus::initial());
    }

    #[test]
    fn connected_shows_everything() {
        let s = status("wlan1", Some(&device(100)), None);
        assert_eq!(s.state, UplinkState::Connected);
        assert_eq!(s.interface.as_deref(), Some("wlan1"));
        assert_eq!(s.ssid.as_deref(), Some("Hotel-Guest"));
        assert_eq!(s.signal, Some(72));
        assert_eq!(s.ipv4, Some(Ipv4Addr::new(192, 0, 2, 15)));
        assert_eq!(s.internet, Some(true));
        assert_eq!(s.error, None);
    }

    #[test]
    fn connecting_has_network_but_no_address_or_check() {
        let s = status("wlan1", Some(&device(70)), None);
        assert_eq!(s.state, UplinkState::Connecting);
        assert_eq!(s.ssid.as_deref(), Some("Hotel-Guest"));
        assert_eq!(s.ipv4, None);
        assert_eq!(s.internet, None);
    }

    #[test]
    fn disconnected_has_no_network_values() {
        let s = status("wlan1", Some(&device(30)), None);
        assert_eq!(s.state, UplinkState::Disconnected);
        assert_eq!(
            (s.ssid, s.signal, s.ipv4, s.internet),
            (None, None, None, None)
        );
    }

    #[test]
    fn failure_is_kept_while_disconnected() {
        let s = status("wlan1", Some(&device(30)), Some(7));
        assert_eq!(s.state, UplinkState::Failed);
        assert_eq!(s.error.as_deref(), Some("wrong or missing password"));
    }

    #[test]
    fn failed_device_without_known_reason_has_an_error() {
        let s = status("wlan1", Some(&device(120)), None);
        assert_eq!(s.state, UplinkState::Failed);
        assert_eq!(s.error.as_deref(), Some("connection failed"));
    }

    #[test]
    fn captive_portal_is_not_internet() {
        let portal = Device {
            connectivity: 2,
            ..device(100)
        };
        assert_eq!(status("wlan1", Some(&portal), None).internet, Some(false));
        let unknown = Device {
            connectivity: CONNECTIVITY_UNKNOWN,
            ..device(100)
        };
        assert_eq!(status("wlan1", Some(&unknown), None).internet, None);
    }

    #[test]
    fn failure_texts_are_own_texts() {
        assert_eq!(failure_text(53), "network not found");
        assert_eq!(failure_text(18), "no address received");
        assert_eq!(failure_text(9), "authentication failed");
        assert_eq!(failure_text(999), "connection failed");
    }

    #[test]
    fn result_fits_the_contract() {
        // The same JSON shape as every other section: all fields present, unknown ones null.
        let value = serde_json::to_value(status("wlan1", Some(&device(30)), None)).unwrap();
        assert_eq!(value["state"], "disconnected");
        assert_eq!(value["ssid"], serde_json::Value::Null);
        assert_eq!(value["interface"], "wlan1");
    }
}
