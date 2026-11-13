//! Status sections as described in `api/openapi.yaml`.
//!
//! Each module reports exactly one section. A section is always sent as a whole, in
//! `GET /v0/status` and in the event stream. `Option` fields are serialized as `null`,
//! never left out.

use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

/// What every section type must provide so that the core can handle it.
pub trait Section: Clone + PartialEq + Serialize + Send + Sync + 'static {
    /// Name of the section in `/v0/status` and of its event in `/v0/events`.
    const NAME: &'static str;

    /// State shown before the module has reported anything.
    fn initial() -> Self;

    /// State shown after the module task has ended unexpectedly: `failed` with
    /// `error` set to [`MODULE_STOPPED`].
    fn stopped(&self) -> Self;
}

/// Error text used when a module task ends.
pub const MODULE_STOPPED: &str = "module stopped";

/// State of the own Wi-Fi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApState {
    Unavailable,
    Starting,
    Running,
    Stopped,
    Failed,
}

/// Section `ap`: the own Wi-Fi that devices connect to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApStatus {
    pub interface: Option<String>,
    pub state: ApState,
    pub ssid: Option<String>,
    pub channel: Option<u32>,
    pub subnet: Option<String>,
    pub clients: Option<u32>,
    pub error: Option<String>,
}

impl Section for ApStatus {
    const NAME: &'static str = "ap";

    fn initial() -> Self {
        ApStatus {
            interface: None,
            state: ApState::Unavailable,
            ssid: None,
            channel: None,
            subnet: None,
            clients: None,
            error: None,
        }
    }

    fn stopped(&self) -> Self {
        ApStatus {
            interface: self.interface.clone(),
            state: ApState::Failed,
            error: Some(MODULE_STOPPED.to_string()),
            ..ApStatus::initial()
        }
    }
}

/// State of the connection to the internet source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UplinkState {
    Unavailable,
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Section `uplink`: the connection to the internet source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UplinkStatus {
    pub interface: Option<String>,
    pub state: UplinkState,
    pub ssid: Option<String>,
    /// Signal strength in percent, 0 to 100.
    pub signal: Option<u8>,
    pub ipv4: Option<Ipv4Addr>,
    /// Result of the connectivity check; `None` while unknown.
    pub internet: Option<bool>,
    pub error: Option<String>,
}

impl Section for UplinkStatus {
    const NAME: &'static str = "uplink";

    fn initial() -> Self {
        UplinkStatus {
            interface: None,
            state: UplinkState::Unavailable,
            ssid: None,
            signal: None,
            ipv4: None,
            internet: None,
            error: None,
        }
    }

    fn stopped(&self) -> Self {
        UplinkStatus {
            interface: self.interface.clone(),
            state: UplinkState::Failed,
            error: Some(MODULE_STOPPED.to_string()),
            ..UplinkStatus::initial()
        }
    }
}

/// Body of `GET /v0/status`: one field per section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub ap: ApStatus,
    pub uplink: UplinkStatus,
}

/// Body of every error response.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: &'static str,
}

#[cfg(test)]
mod tests {
    //! Checks the types against the examples in `api/openapi.yaml`, so that code and
    //! contract cannot drift apart unnoticed.

    use super::*;
    use serde_json::Value;

    const OPENAPI: &str = include_str!("../../../api/openapi.yaml");

    fn status_examples() -> Vec<(String, Value)> {
        let doc: Value = serde_norway::from_str(OPENAPI).expect("openapi.yaml is valid YAML");
        let examples = &doc["paths"]["/v0/status"]["get"]["responses"]["200"]["content"]["application/json"]
            ["examples"];
        examples
            .as_object()
            .expect("GET /v0/status has examples")
            .iter()
            .map(|(name, example)| (name.clone(), example["value"].clone()))
            .collect()
    }

    #[test]
    fn status_examples_round_trip() {
        let examples = status_examples();
        assert!(!examples.is_empty());
        for (name, example) in examples {
            let parsed: Status = serde_json::from_value(example.clone())
                .unwrap_or_else(|e| panic!("example {name} does not fit the types: {e}"));
            let back = serde_json::to_value(&parsed).unwrap();
            assert_eq!(back, example, "example {name} changes in a round trip");
        }
    }

    #[test]
    fn required_fields_match_the_contract() {
        let doc: Value = serde_norway::from_str(OPENAPI).unwrap();
        let schemas = &doc["components"]["schemas"];
        let cases = [
            (
                "ApStatus",
                serde_json::to_value(ApStatus::initial()).unwrap(),
            ),
            (
                "UplinkStatus",
                serde_json::to_value(UplinkStatus::initial()).unwrap(),
            ),
        ];
        for (schema, value) in cases {
            let mut required: Vec<&str> = schemas[schema]["required"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            let mut fields: Vec<&str> = value
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            required.sort();
            fields.sort();
            assert_eq!(fields, required, "fields of {schema}");
        }
    }

    #[test]
    fn missing_values_are_null() {
        let value = serde_json::to_value(UplinkStatus::initial()).unwrap();
        assert_eq!(value["ssid"], Value::Null);
        assert_eq!(value["state"], "unavailable");
    }

    #[test]
    fn stopped_keeps_interface_and_sets_error() {
        let running = ApStatus {
            interface: Some("wlan0".into()),
            state: ApState::Running,
            ssid: Some("TravelPi".into()),
            channel: Some(6),
            subnet: Some("10.157.1.0/24".into()),
            clients: Some(2),
            error: None,
        };
        let stopped = running.stopped();
        assert_eq!(stopped.interface.as_deref(), Some("wlan0"));
        assert_eq!(stopped.state, ApState::Failed);
        assert_eq!(stopped.error.as_deref(), Some(MODULE_STOPPED));
    }
}
