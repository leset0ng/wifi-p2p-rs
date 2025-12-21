use std::collections::HashMap;

use zbus::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::error::P2pError;

use super::{P2pBackend, P2pFuture};

const WPA_SUPPLICANT_DEST: &str = "fi.w1.wpa_supplicant1";
const WPA_SUPPLICANT_PATH: &str = "/fi/w1/wpa_supplicant1";
const WPA_SUPPLICANT_IFACE: &str = "fi.w1.wpa_supplicant1";
const WPA_SUPPLICANT_P2P_IFACE: &str = "fi.w1.wpa_supplicant1.Interface.P2PDevice";

#[derive(Debug, Clone)]
pub struct P2pBackendImpl {
    connection: Connection,
    interface_path: OwnedObjectPath,
}

impl P2pBackendImpl {
    /// Build a backend by resolving the interface object path
    /// from wpa_supplicant using the provided interface name (e.g. "wlan0").
    pub async fn new(connection: &Connection, interface_name: &str) -> Result<Self, P2pError> {
        if interface_name.trim().is_empty() {
            return Err(P2pError::InvalidInterface(interface_name.to_string()));
        }
        let interface_path = Self::get_interface_path(connection, interface_name).await?;
        Ok(Self {
            connection: connection.clone(),
            interface_path,
        })
    }

    async fn get_interface_path(
        connection: &Connection,
        interface_name: &str,
    ) -> Result<OwnedObjectPath, P2pError> {
        let proxy =
            zbus::Proxy::new(connection, WPA_SUPPLICANT_DEST, WPA_SUPPLICANT_PATH, WPA_SUPPLICANT_IFACE)
                .await?;

        // The wpa_supplicant root object exposes GetInterface(ifname) -> object path.
        let path: OwnedObjectPath = proxy.call("GetInterface", &(interface_name)).await?;
        Ok(path)
    }

    async fn p2p_proxy(&self) -> Result<zbus::Proxy<'_>, P2pError> {
        // Create a fresh proxy per call to avoid lifetime gymnastics and
        // keep each operation independent (important for async call ordering).
        let proxy = zbus::Proxy::new(
            &self.connection,
            WPA_SUPPLICANT_DEST,
            self.interface_path.clone(),
            WPA_SUPPLICANT_P2P_IFACE,
        )
        .await?;
        Ok(proxy)
    }

    fn empty_options() -> HashMap<String, OwnedValue> {
        // Most P2P D-Bus methods accept a{sv} options; this starts with defaults.
        HashMap::new()
    }
}

impl P2pBackend for P2pBackendImpl {
    fn discover_peers(&self) -> P2pFuture<'_, ()> {
        Box::pin(async move {
            let proxy = self.p2p_proxy().await?;
            // Maps to p2p_find; options follow wpa_supplicant's a{sv} signature.
            let options = Self::empty_options();
            let _: () = proxy.call("Find", &(options)).await?;
            Ok(())
        })
    }

    fn stop_discovery(&self) -> P2pFuture<'_, ()> {
        Box::pin(async move {
            let proxy = self.p2p_proxy().await?;
            // Maps to p2p_stop_find.
            let _: () = proxy.call("StopFind", &()).await?;
            Ok(())
        })
    }

    fn connect(&self, device_address: String) -> P2pFuture<'_, ()> {
        Box::pin(async move {
            let proxy = self.p2p_proxy().await?;
            // Maps to p2p_connect. Adjust option keys to match your wpa_supplicant build.
            // Some builds expect "peer" as an object path; others accept the MAC address.
            let mut options = Self::empty_options();
            let peer = OwnedValue::try_from(Value::from(device_address))?;
            let wps = OwnedValue::try_from(Value::from("pbc"))?;
            options.insert("peer".to_string(), peer);
            options.insert("wps_method".to_string(), wps);
            let _: () = proxy.call("Connect", &(options)).await?;
            Ok(())
        })
    }

    fn create_group(&self) -> P2pFuture<'_, ()> {
        Box::pin(async move {
            let proxy = self.p2p_proxy().await?;
            // Maps to p2p_group_add.
            let options = Self::empty_options();
            let _: () = proxy.call("GroupAdd", &(options)).await?;
            Ok(())
        })
    }
}
