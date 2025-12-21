#[derive(Debug, Clone)]
pub struct P2pDevice {
    /// Device MAC address (e.g. "02:11:22:33:44:55").
    pub mac_address: String,
    /// Optional device name reported by P2P.
    pub device_name: Option<String>,
    /// Optional primary device type (e.g. "1-0050F204-1").
    pub primary_type: Option<String>,
}
