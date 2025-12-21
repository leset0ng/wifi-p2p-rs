use tokio::sync::{broadcast, mpsc, oneshot};

use crate::device::P2pDevice;
use crate::error::P2pError;
use crate::manager::ManagerCommand;

pub type ActionReceiver = oneshot::Receiver<Result<(), P2pError>>;

#[derive(Debug, Clone)]
pub enum P2pEvent {
    /// Local discovery request succeeded and the scan is active.
    DiscoveryStarted,
    /// Local request to stop discovery succeeded.
    DiscoveryStopped,
    /// Local request to form a group succeeded.
    GroupCreated,
    /// Local connect request succeeded for the given peer address.
    Connected(String),
    /// Placeholder event for peer detection (would be driven by D-Bus signals).
    PeerFound(P2pDevice),
}

#[derive(Clone)]
pub struct WifiP2pChannel {
    command_tx: mpsc::Sender<ManagerCommand>,
    event_tx: broadcast::Sender<P2pEvent>,
}

impl WifiP2pChannel {
    pub(crate) fn new(
        command_tx: mpsc::Sender<ManagerCommand>,
        event_tx: broadcast::Sender<P2pEvent>,
    ) -> Self {
        Self { command_tx, event_tx }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<P2pEvent> {
        // Each subscriber gets its own receiver, similar to Android's intent listeners.
        self.event_tx.subscribe()
    }

    pub async fn discover_peers(&self) -> Result<ActionReceiver, P2pError> {
        // This mirrors ActionListener by returning a oneshot channel for the result.
        let (respond_to, receiver) = oneshot::channel();
        self.send_command(ManagerCommand::Discover { respond_to }).await?;
        Ok(receiver)
    }

    pub async fn stop_discovery(&self) -> Result<ActionReceiver, P2pError> {
        // Stop discovery and report completion through the oneshot.
        let (respond_to, receiver) = oneshot::channel();
        self.send_command(ManagerCommand::StopDiscovery { respond_to })
            .await?;
        Ok(receiver)
    }

    pub async fn connect(&self, device_address: String) -> Result<ActionReceiver, P2pError> {
        // Queue a connect command; the worker does the D-Bus call.
        let (respond_to, receiver) = oneshot::channel();
        self.send_command(ManagerCommand::Connect {
            device_address,
            respond_to,
        })
        .await?;
        Ok(receiver)
    }

    pub async fn create_group(&self) -> Result<ActionReceiver, P2pError> {
        // Create a P2P group with default options.
        let (respond_to, receiver) = oneshot::channel();
        self.send_command(ManagerCommand::CreateGroup { respond_to })
            .await?;
        Ok(receiver)
    }

    async fn send_command(&self, command: ManagerCommand) -> Result<(), P2pError> {
        // If the manager task is gone, convert it into a typed error.
        self.command_tx
            .send(command)
            .await
            .map_err(|_| P2pError::ChannelClosed("manager".to_string()))
    }
}
