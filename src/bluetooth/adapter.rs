use anyhow::Result;
use bluer::{Adapter as BluerAdapter, Address};
use futures::StreamExt;
use log::{debug, info, warn};

use crate::bluetooth::device::BluetoothDevice;

#[derive(Debug, Clone)]
pub struct Adapter {
    adapter: BluerAdapter,
    name: String,
}

impl Adapter {
    pub async fn new(adapter: BluerAdapter, name: String) -> Result<Self> {
        Ok(Self {
            adapter,
            name,
        })
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub async fn is_powered(&self) -> Result<bool> {
        Ok(self.adapter.is_powered().await?)
    }
    
    pub async fn is_discoverable(&self) -> Result<bool> {
        Ok(self.adapter.is_discoverable().await?)
    }
    
    pub async fn is_pairable(&self) -> Result<bool> {
        Ok(self.adapter.is_pairable().await?)
    }
    
    pub async fn set_powered(&self, powered: bool) -> Result<()> {
        info!("Setting adapter power: {}", powered);
        self.adapter.set_powered(powered).await?;
        Ok(())
    }
    
    pub async fn set_discoverable(&self, discoverable: bool) -> Result<()> {
        info!("Setting adapter discoverable: {}", discoverable);
        self.adapter.set_discoverable(discoverable).await?;
        Ok(())
    }
    
    pub async fn set_pairable(&self, pairable: bool) -> Result<()> {
        info!("Setting adapter pairable: {}", pairable);
        self.adapter.set_pairable(pairable).await?;
        Ok(())
    }
    
    pub async fn start_discovery(&self) -> Result<()> {
        info!("Starting device discovery");
        // Start discovery and consume the stream
        let mut stream = self.adapter.discover_devices().await?;
        // Start the discovery but don't block - just consume the first item to initiate
        tokio::spawn(async move {
            while let Some(_event) = stream.next().await {
                // Discovery events are handled, devices will appear in device_addresses()
            }
        });
        Ok(())
    }
    
    pub async fn stop_discovery(&self) -> Result<()> {
        info!("Stopping device discovery");
        // Note: bluer doesn't have explicit stop_discovery, it handles this automatically
        Ok(())
    }
    
    pub async fn get_devices(&self) -> Result<Vec<BluetoothDevice>> {
        debug!("Getting known devices");
        let mut devices = Vec::new();
        
        // Get device addresses from the adapter
        let device_addresses = self.adapter.device_addresses().await?;
        debug!("Found {} device addresses", device_addresses.len());
        
        for address in device_addresses {
            debug!("Processing device: {}", address);
            match self.adapter.device(address) {
                Ok(device) => {
                    // Get device properties
                    let name = device.name().await.unwrap_or(None);
                    let connected = device.is_connected().await.unwrap_or(false);
                    let paired = device.is_paired().await.unwrap_or(false);
                    let trusted = device.is_trusted().await.unwrap_or(false);
                    let rssi = device.rssi().await.ok().flatten();
                    let uuids = match device.uuids().await {
                        Ok(uuid_set) => uuid_set.into_iter()
                            .map(|uuid| format!("{:?}", uuid))
                            .collect::<Vec<String>>(),
                        Err(_) => Vec::new(),
                    };
                    
                    let mut bluetooth_device = BluetoothDevice {
                        address: address.to_string(),
                        name: name.unwrap_or_else(|| "Unknown Device".to_string()),
                        device_type: "Unknown".to_string(),
                        connected,
                        paired,
                        trusted,
                        rssi,
                        uuids,
                    };
                    
                    // Update device type based on UUIDs
                    bluetooth_device.update_device_type();
                    debug!("Added device: {} ({})", bluetooth_device.name, address);
                    devices.push(bluetooth_device);
                }
                Err(e) => {
                    warn!("Failed to get device {}: {}", address, e);
                }
            }
        }
        
        debug!("Found {} devices total", devices.len());
        Ok(devices)
    }
    
    pub async fn connect_device(&self, address: Address) -> Result<()> {
        info!("Connecting to device: {}", address);
        let device = self.adapter.device(address)?;
        device.connect().await?;
        Ok(())
    }
    
    pub async fn disconnect_device(&self, address: Address) -> Result<()> {
        info!("Disconnecting from device: {}", address);
        let device = self.adapter.device(address)?;
        device.disconnect().await?;
        Ok(())
    }
    
    pub async fn pair_device(&self, address: Address) -> Result<()> {
        info!("Pairing with device: {}", address);
        let device = self.adapter.device(address)?;
        device.pair().await?;
        Ok(())
    }
    
    pub async fn remove_device(&self, address: Address) -> Result<()> {
        info!("Removing device: {}", address);
        self.adapter.remove_device(address).await?;
        Ok(())
    }
}
