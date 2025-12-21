use std::future::Future;
use std::pin::Pin;

use crate::error::P2pError;

pub type P2pFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, P2pError>> + Send + 'a>>;

pub trait P2pBackend: Send + Sync {
    /// Start a peer discovery scan (maps to p2p_find).
    fn discover_peers(&self) -> P2pFuture<'_, ()>;
    /// Stop the ongoing peer discovery scan (maps to p2p_stop_find).
    fn stop_discovery(&self) -> P2pFuture<'_, ()>;
    /// Connect to a peer by device address (maps to p2p_connect).
    fn connect(&self, device_address: String) -> P2pFuture<'_, ()>;
    /// Create a P2P group (maps to p2p_group_add).
    fn create_group(&self) -> P2pFuture<'_, ()>;
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::P2pBackendImpl;

#[cfg(not(target_os = "linux"))]
compile_error!("Only Linux is supported right now. Add a platform backend for this target.");
