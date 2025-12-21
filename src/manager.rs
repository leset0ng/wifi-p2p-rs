use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot};
use zbus::Connection;

use crate::backend::{P2pBackend, P2pBackendImpl};
use crate::channel::{P2pEvent, WifiP2pChannel};
use crate::error::P2pError;

pub struct WifiP2pManager {
    connection: Connection,
    backend: Arc<dyn P2pBackend>,
}

impl WifiP2pManager {
    /// Build the manager and its Linux backend by opening the system bus
    /// and resolving the wpa_supplicant interface object path.
    pub async fn new(interface_name: &str) -> Result<Self, P2pError> {
        let connection = Connection::system().await?;
        let backend = P2pBackendImpl::new(&connection, interface_name).await?;
        Ok(Self {
            connection,
            backend: Arc::new(backend),
        })
    }

    pub fn initialize(&self) -> WifiP2pChannel {
        // The channel owns the command sender; a background task consumes commands
        // and executes D-Bus calls on the backend.
        let (command_tx, command_rx) = mpsc::channel(32);
        let (event_tx, _event_rx) = broadcast::channel(64);
        let event_tx_for_task = event_tx.clone();
        let backend = Arc::clone(&self.backend);
        tokio::spawn(async move {
            run_manager(backend, command_rx, event_tx_for_task).await;
        });
        WifiP2pChannel::new(command_tx, event_tx)
    }

    pub fn connection(&self) -> &Connection {
        // Expose the raw connection for advanced consumers (signals, extra interfaces).
        &self.connection
    }
}

pub(crate) enum ManagerCommand {
    Discover {
        respond_to: oneshot::Sender<Result<(), P2pError>>,
    },
    StopDiscovery {
        respond_to: oneshot::Sender<Result<(), P2pError>>,
    },
    Connect {
        device_address: String,
        respond_to: oneshot::Sender<Result<(), P2pError>>,
    },
    CreateGroup {
        respond_to: oneshot::Sender<Result<(), P2pError>>,
    },
}

async fn run_manager(
    backend: Arc<dyn P2pBackend>,
    mut command_rx: mpsc::Receiver<ManagerCommand>,
    event_tx: broadcast::Sender<P2pEvent>,
) {
    // Single consumer loop that serializes backend operations to avoid
    // overlapping D-Bus requests unless explicitly desired.
    while let Some(command) = command_rx.recv().await {
        match command {
            ManagerCommand::Discover { respond_to } => {
                let result = backend.discover_peers().await;
                if result.is_ok() {
                    let _ = event_tx.send(P2pEvent::DiscoveryStarted);
                }
                let _ = respond_to.send(result);
            }
            ManagerCommand::StopDiscovery { respond_to } => {
                let result = backend.stop_discovery().await;
                if result.is_ok() {
                    let _ = event_tx.send(P2pEvent::DiscoveryStopped);
                }
                let _ = respond_to.send(result);
            }
            ManagerCommand::Connect {
                device_address,
                respond_to,
            } => {
                let event_address = device_address.clone();
                let result = backend.connect(device_address).await;
                if result.is_ok() {
                    let _ = event_tx.send(P2pEvent::Connected(event_address));
                }
                let _ = respond_to.send(result);
            }
            ManagerCommand::CreateGroup { respond_to } => {
                let result = backend.create_group().await;
                if result.is_ok() {
                    let _ = event_tx.send(P2pEvent::GroupCreated);
                }
                let _ = respond_to.send(result);
            }
        }
    }
}
