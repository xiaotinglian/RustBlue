use bluer::Address;

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub device_type: String,
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
    pub rssi: Option<i16>,
    pub uuids: Vec<String>,
}

impl BluetoothDevice {
    pub fn new(address: Address, name: Option<String>) -> Self {
        Self {
            address: address.to_string(),
            name: name.unwrap_or_else(|| "Unknown Device".to_string()),
            device_type: "Unknown Device".to_string(),
            connected: false,
            paired: false,
            trusted: false,
            rssi: None,
            uuids: Vec::new(),
        }
    }
    
    pub fn update_device_type(&mut self) {
        self.device_type = Self::determine_device_type(&self.uuids);
    }
    
    pub fn new_test(name: &str, address: &str, connected: bool) -> Self {
        Self {
            address: address.to_string(),
            name: name.to_string(),
            device_type: "Test Device".to_string(),
            connected,
            paired: false,
            trusted: false,
            rssi: Some(-50),
            uuids: Vec::new(),
        }
    }
    
    pub fn address(&self) -> String {
        self.address.to_string()
    }
    
    pub fn name(&self) -> String {
        self.name.clone()
    }
    
    pub fn device_type(&self) -> String {
        self.device_type.clone()
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    pub fn is_paired(&self) -> bool {
        self.paired
    }
    
    pub fn is_trusted(&self) -> bool {
        self.trusted
    }
    
    pub fn rssi(&self) -> Option<i16> {
        self.rssi
    }
    
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }
    
    pub fn set_paired(&mut self, paired: bool) {
        self.paired = paired;
    }
    
    pub fn set_trusted(&mut self, trusted: bool) {
        self.trusted = trusted;
    }
    
    fn determine_device_type(uuids: &[String]) -> String {
        // Basic device type detection based on UUIDs
        // This is a simplified version - in reality, you'd check against known service UUIDs
        
        for uuid in uuids {
            // Audio devices
            if uuid.contains("1108") || uuid.contains("110b") || uuid.contains("110d") {
                return "Audio Device".to_string();
            }
            
            // HID devices
            if uuid.contains("1124") {
                return "Input Device".to_string();
            }
            
            // Network
            if uuid.contains("1115") || uuid.contains("1116") {
                return "Network Device".to_string();
            }
            
            // File transfer
            if uuid.contains("1105") || uuid.contains("1106") {
                return "File Transfer".to_string();
            }
        }
        
        "Unknown Device".to_string()
    }
}
