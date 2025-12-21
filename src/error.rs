use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    /// A transport or method call error from the D-Bus layer.
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),
    /// Serialization/deserialization failures for D-Bus values.
    #[error("D-Bus serialization error: {0}")]
    ZVariant(#[from] zbus::zvariant::Error),
    /// The async command channel closed unexpectedly.
    #[error("channel closed: {0}")]
    ChannelClosed(String),
    /// Invalid or empty interface name provided by the caller.
    #[error("invalid interface name: {0}")]
    InvalidInterface(String),
    /// Other backend-specific errors not mapped above.
    #[error("backend error: {0}")]
    Backend(String),
}
