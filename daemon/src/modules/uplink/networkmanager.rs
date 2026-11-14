//! Reads the uplink device from NetworkManager over D-Bus (system bus).
//!
//! Only reading: which state the device is in, which network, how strong, which address,
//! whether the internet is reachable. Connecting to a network comes with the commands of the
//! API (TRA-33, TRA-35); until then the connection is a stored NetworkManager profile.
//!
//! D-Bus interfaces used (all from `org.freedesktop.NetworkManager`):
//! https://networkmanager.dev/docs/api/latest/spec.html

use std::collections::HashMap;
use std::future::Future;
use std::net::Ipv4Addr;
use std::time::Duration;

use tokio::time::{sleep, timeout};
use tokio_stream::StreamExt;
use zbus::proxy::{Builder, CacheProperties};
use zbus::{Connection, Proxy, proxy::SignalStream};
use zbus::{OwnedObjectPath, OwnedVariant};

use super::state::Device;

const SERVICE: &str = "org.freedesktop.NetworkManager";
const MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";
const MANAGER: &str = "org.freedesktop.NetworkManager";
const DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const ACCESS_POINT: &str = "org.freedesktop.NetworkManager.AccessPoint";
const IP4_CONFIG: &str = "org.freedesktop.NetworkManager.IP4Config";
/// Error NetworkManager returns if no device has this interface name.
const UNKNOWN_DEVICE: &str = "org.freedesktop.NetworkManager.UnknownDevice";
/// Object path NetworkManager uses for "none".
const NONE: &str = "/";

/// A change of the device state, as sent by NetworkManager in the signal `StateChanged`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Change {
    pub state: u32,
    /// Why the state changed (`NMDeviceStateReason`), for example 7 for "no secrets".
    pub reason: u32,
}

/// Where the module gets the device from. NetworkManager on the Pi; a stand-in in tests.
pub trait Source: Send {
    /// Reads the device with this interface name. `Ok(None)` if there is no such device.
    fn read(
        &mut self,
        interface: &str,
    ) -> impl Future<Output = Result<Option<Device>, String>> + Send;

    /// Waits until the device state changes, but at most `limit`.
    fn wait_for_change(&mut self, limit: Duration) -> impl Future<Output = Option<Change>> + Send;
}

/// NetworkManager on the system bus.
#[derive(Default)]
pub struct NetworkManager {
    connection: Option<Connection>,
    /// Device whose `StateChanged` signal is being received, and the signal stream.
    watched: Option<(OwnedObjectPath, SignalStream<'static>)>,
}

impl Source for NetworkManager {
    async fn read(&mut self, interface: &str) -> Result<Option<Device>, String> {
        let result = self.read_device(interface).await;
        if result.is_err() {
            // Start over with a new connection next time, NetworkManager may have restarted.
            self.connection = None;
            self.watched = None;
        }
        result.map_err(|error| error.to_string())
    }

    async fn wait_for_change(&mut self, limit: Duration) -> Option<Change> {
        let Some((_, signals)) = &mut self.watched else {
            sleep(limit).await;
            return None;
        };
        match timeout(limit, signals.next()).await {
            Ok(Some(message)) => {
                // StateChanged(new state, old state, reason)
                let (state, _old, reason): (u32, u32, u32) = message.body().deserialize().ok()?;
                Some(Change { state, reason })
            }
            Ok(None) => {
                self.watched = None;
                None
            }
            Err(_elapsed) => None,
        }
    }
}

impl NetworkManager {
    async fn read_device(&mut self, interface: &str) -> zbus::Result<Option<Device>> {
        let connection = match &self.connection {
            Some(connection) => connection.clone(),
            None => {
                let connection = Connection::system().await?;
                self.connection = Some(connection.clone());
                connection
            }
        };

        let manager = proxy(&connection, MANAGER_PATH.to_string(), MANAGER).await?;
        let path: OwnedObjectPath = match manager.call("GetDeviceByIpIface", &(interface,)).await {
            Ok(path) => path,
            Err(zbus::Error::MethodError(name, _, _)) if name.as_str() == UNKNOWN_DEVICE => {
                self.watched = None;
                return Ok(None);
            }
            Err(error) => return Err(error),
        };

        let device = proxy(&connection, path.to_string(), DEVICE).await?;
        if self.watched.as_ref().map(|(watched, _)| watched) != Some(&path) {
            let signals = device.receive_signal("StateChanged").await?;
            self.watched = Some((path.clone(), signals));
        }

        let state: u32 = device.get_property("State").await?;
        let connectivity: u32 = device.get_property("Ip4Connectivity").await?;

        let wireless = proxy(&connection, path.to_string(), WIRELESS).await?;
        let access_point: OwnedObjectPath = wireless.get_property("ActiveAccessPoint").await?;
        let (ssid, strength) = if access_point.as_str() == NONE {
            (None, None)
        } else {
            let ap = proxy(&connection, access_point.to_string(), ACCESS_POINT).await?;
            let ssid: Vec<u8> = ap.get_property("Ssid").await?;
            let strength: u8 = ap.get_property("Strength").await?;
            (
                Some(String::from_utf8_lossy(&ssid).into_owned()),
                Some(strength),
            )
        };

        let ip4_config: OwnedObjectPath = device.get_property("Ip4Config").await?;
        let ipv4 = if ip4_config.as_str() == NONE {
            None
        } else {
            let config = proxy(&connection, ip4_config.to_string(), IP4_CONFIG).await?;
            let addresses: Vec<HashMap<String, OwnedVariant>> =
                config.get_property("AddressData").await?;
            first_address(&addresses)
        };

        Ok(Some(Device {
            state,
            ssid,
            strength,
            ipv4,
            connectivity,
        }))
    }
}

/// A proxy that always asks NetworkManager, without a cache, so every read is current.
async fn proxy(
    connection: &Connection,
    path: String,
    interface: &'static str,
) -> zbus::Result<Proxy<'static>> {
    Builder::<Proxy>::new(connection)
        .destination(SERVICE)
        .path(path)
        .interface(interface)
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

/// `AddressData` is a list of entries like `{"address": "192.0.2.15", "prefix": 24}`.
fn first_address(addresses: &[HashMap<String, OwnedVariant>]) -> Option<Ipv4Addr> {
    addresses
        .iter()
        .filter_map(|entry| entry.get("address"))
        .filter_map(|value| <&str>::try_from(value).ok())
        .find_map(|text| text.parse().ok())
}
