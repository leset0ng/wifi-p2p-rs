use wifi_direct_rs::{P2pEvent, WifiP2pManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace "wlan0" with the interface that wpa_supplicant manages on your system.
    let manager = WifiP2pManager::new("wlan0").await?;
    let channel = manager.initialize();

    // Subscribe to high-level events (simulating Android's broadcast intents).
    let mut events = channel.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                P2pEvent::DiscoveryStarted => {
                    println!("P2P discovery started");
                }
                P2pEvent::DiscoveryStopped => {
                    println!("P2P discovery stopped");
                }
                P2pEvent::GroupCreated => {
                    println!("P2P group created");
                }
                P2pEvent::Connected(addr) => {
                    println!("Connected to peer {addr}");
                }
                P2pEvent::PeerFound(device) => {
                    println!(
                        "Peer found: {} ({:?})",
                        device.mac_address, device.device_name
                    );
                }
            }
        }
    });

    // Trigger discovery and wait for the action result.
    let action = channel.discover_peers().await?;
    action.await??;

    Ok(())
}
