use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use bluer::{Address, Session};
use log::{debug, error, info, warn};
use tokio::sync::RwLock;

use crate::bluetooth::adapter::Adapter;
use crate::bluetooth::device::BluetoothDevice;

#[derive(Debug)]
pub struct BluetoothManager {
    session: Session,
    adapters: Arc<RwLock<HashMap<String, Adapter>>>,
    default_adapter: Arc<RwLock<Option<String>>>,
}

impl BluetoothManager {
    pub async fn new() -> Result<Self> {
        info!("Initializing Bluetooth manager");
        
        let session = Session::new().await?;
        let adapters = Arc::new(RwLock::new(HashMap::new()));
        let default_adapter = Arc::new(RwLock::new(None));
        
        let manager = Self {
            session,
            adapters,
            default_adapter,
        };
        
        manager.discover_adapters().await?;
        
        Ok(manager)
    }
    
    pub async fn discover_adapters(&self) -> Result<()> {
        debug!("Discovering Bluetooth adapters");
        
        let adapter_names = self.session.adapter_names().await?;
        let mut adapters = self.adapters.write().await;
        let mut default_adapter = self.default_adapter.write().await;
        
        adapters.clear();
        
        for name in adapter_names {
                match self.session.adapter(&name) {
                    Ok(bluer_adapter) => {
                        match Adapter::new(bluer_adapter, name.clone()).await {
                            Ok(adapter) => {
                                info!("Found adapter: {}", adapter.name());
                                
                                // Set first adapter as default if none is set
                                if default_adapter.is_none() {
                                    *default_adapter = Some(name.clone());
                                    info!("Set default adapter: {}", name);
                                }
                                
                                adapters.insert(name, adapter);
                            }
                            Err(e) => {
                                error!("Failed to initialize adapter {}: {}", name, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get adapter {}: {}", name, e);
                    }
                }
        }
        
        info!("Discovered {} adapters", adapters.len());
        Ok(())
    }
    
    pub async fn get_default_adapter(&self) -> Option<Adapter> {
        let default_name = self.default_adapter.read().await;
        if let Some(name) = default_name.as_ref() {
            let adapters = self.adapters.read().await;
            adapters.get(name).cloned()
        } else {
            None
        }
    }
    
    pub async fn get_adapter(&self, name: &str) -> Option<Adapter> {
        let adapters = self.adapters.read().await;
        adapters.get(name).cloned()
    }
    
    pub async fn list_adapters(&self) -> Vec<String> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }
    
    pub async fn start_discovery(&self) -> Result<()> {
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.start_discovery().await?;
        } else {
            warn!("No default adapter available for discovery");
        }
        Ok(())
    }
    
    pub async fn stop_discovery(&self) -> Result<()> {
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.stop_discovery().await?;
        } else {
            warn!("No default adapter available to stop discovery");
        }
        Ok(())
    }
    
    pub async fn get_devices(&self) -> Result<Vec<BluetoothDevice>> {
        debug!("Getting known devices");
        let mut devices = Vec::new();
        
        let default_adapter_name = self.default_adapter.read().await;
        if let Some(adapter_name) = default_adapter_name.as_ref() {
            let adapters = self.adapters.read().await;
            if let Some(adapter) = adapters.get(adapter_name) {
                devices = adapter.get_devices().await?;
            }
        }
        
        debug!("Found {} devices", devices.len());
        Ok(devices)
    }
    
    pub async fn connect_device(&self, address: &str) -> Result<()> {
        let addr: Address = address.parse()?;
        
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.connect_device(addr).await?;
        } else {
            return Err(anyhow::anyhow!("No default adapter available"));
        }
        
        Ok(())
    }
    
    pub async fn disconnect_device(&self, address: &str) -> Result<()> {
        let addr: Address = address.parse()?;
        
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.disconnect_device(addr).await?;
        } else {
            return Err(anyhow::anyhow!("No default adapter available"));
        }
        
        Ok(())
    }
    
    pub async fn pair_device(&self, address: &str) -> Result<()> {
        let addr: Address = address.parse()?;
        
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.pair_device(addr).await?;
        } else {
            return Err(anyhow::anyhow!("No default adapter available"));
        }
        
        Ok(())
    }
    
    pub async fn remove_device(&self, address: &str) -> Result<()> {
        let addr: Address = address.parse()?;
        
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.remove_device(addr).await?;
        } else {
            return Err(anyhow::anyhow!("No default adapter available"));
        }
        
        Ok(())
    }
    
    pub async fn set_adapter_powered(&self, powered: bool) -> Result<()> {
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.set_powered(powered).await?;
        } else {
            return Err(anyhow::anyhow!("No default adapter available"));
        }
        
        Ok(())
    }
    
    pub async fn set_adapter_discoverable(&self, discoverable: bool) -> Result<()> {
        if let Some(adapter) = self.get_default_adapter().await {
            adapter.set_discoverable(discoverable).await?;
        } else {
            return Err(anyhow::anyhow!("No default adapter available"));
        }
        
        Ok(())
    }
}
