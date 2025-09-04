use gtk::{
    glib, prelude::*, subclass::prelude::*, Application, ApplicationWindow, Box as GtkBox,
    Button, HeaderBar, Label, Orientation, Switch,
};

use crate::bluetooth::manager::BluetoothManager;
use crate::ui::device_list::DeviceListView;

glib::wrapper! {
    pub struct RustBlueWindow(ObjectSubclass<imp::RustBlueWindow>)
        @extends ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RustBlueWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }
}

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct RustBlueWindow {
        pub device_list: RefCell<Option<DeviceListView>>,
        pub bluetooth_manager: RefCell<Option<BluetoothManager>>,
        pub scan_button: RefCell<Option<Button>>,
        pub bluetooth_toggle: RefCell<Option<Switch>>,
        pub auto_scan_source_id: RefCell<Option<glib::SourceId>>,
    }

    impl Default for RustBlueWindow {
        fn default() -> Self {
            Self {
                device_list: RefCell::new(None),
                bluetooth_manager: RefCell::new(None),
                scan_button: RefCell::new(None),
                bluetooth_toggle: RefCell::new(None),
                auto_scan_source_id: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RustBlueWindow {
        const NAME: &'static str = "RustBlueWindow";
        type Type = super::RustBlueWindow;
        type ParentType = ApplicationWindow;
    }

    impl ObjectImpl for RustBlueWindow {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.setup_ui();
        }
    }

    impl WidgetImpl for RustBlueWindow {}
    impl WindowImpl for RustBlueWindow {}
    impl ApplicationWindowImpl for RustBlueWindow {}
}

impl RustBlueWindow {
    fn setup_ui(&self) {
        // Set window properties for Hyprland compatibility
        self.set_title(Some("RustBlue"));
        self.set_default_size(800, 600);
        self.set_resizable(true);
        
        // Set window position to center (Hyprland will respect this)
        self.set_default_size(800, 600);
        
        // Set window type hint for better Hyprland handling
        self.set_modal(false);
        
        // Create header bar
        let header_bar = HeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("RustBlue"))));
        
        // Add Bluetooth toggle switch to header bar
        let bluetooth_toggle = Switch::new();
        bluetooth_toggle.set_tooltip_text(Some("Turn Bluetooth on/off"));
        bluetooth_toggle.set_active(true); // Assume Bluetooth is on by default
        header_bar.pack_end(&bluetooth_toggle);
        
        let scan_button = Button::with_label("Scan");
        scan_button.set_tooltip_text(Some("Scan for devices"));
        header_bar.pack_start(&scan_button);
        
        self.set_titlebar(Some(&header_bar));
        
        // Initialize Bluetooth manager asynchronously
        let window_weak = self.downgrade();
        glib::spawn_future_local(async move {
            if let Some(window) = window_weak.upgrade() {
                window.initialize_bluetooth().await;
            }
        });
        
        // Create main layout
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);
        
        // Add a visible test label at the top
        let test_label = Label::new(Some("🔍 Bluetooth Manager - Devices should appear below"));
        test_label.set_markup("<big><b>🔍 Bluetooth Manager - Devices should appear below</b></big>");
        test_label.set_halign(gtk::Align::Center);
        test_label.set_margin_top(20);
        test_label.set_margin_bottom(20);
        main_box.append(&test_label);
        
        // Create a simple vertical layout for the device list only
        let content_box = GtkBox::new(Orientation::Vertical, 12);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(12);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        content_box.set_hexpand(true);
        content_box.set_vexpand(true);
        
        // Device list taking the full width
        let device_list = DeviceListView::new();
        device_list.set_hexpand(true);
        device_list.set_vexpand(true);
        
        // Add device list to the content box
        content_box.append(&device_list);
        
        main_box.append(&content_box);
        
        // Set main content
        self.set_child(Some(&main_box));
        
        // Additional window properties for better Hyprland handling
        // Set minimum and maximum size constraints
        self.set_size_request(600, 400);  // Minimum size
        
        // Set the window to be deletable and decorated
        self.set_deletable(true);
        
        // Store references
        let imp = self.imp();
        imp.device_list.replace(Some(device_list));
        imp.scan_button.replace(Some(scan_button.clone()));
        imp.bluetooth_toggle.replace(Some(bluetooth_toggle.clone()));
        
        // Connect signals
        self.connect_signals();
    }
    
    fn connect_signals(&self) {
        let imp = self.imp();
        
        // Set up device list callbacks
        if let Some(device_list) = imp.device_list.borrow().as_ref() {
            let window_weak = self.downgrade();
            device_list.set_connect_callback(move |address| {
                if let Some(window) = window_weak.upgrade() {
                    glib::spawn_future_local(async move {
                        window.connect_device(address).await;
                    });
                }
            });
            
            let window_weak = self.downgrade();
            device_list.set_disconnect_callback(move |address| {
                if let Some(window) = window_weak.upgrade() {
                    glib::spawn_future_local(async move {
                        window.disconnect_device(address).await;
                    });
                }
            });
            
            let window_weak = self.downgrade();
            device_list.set_forget_callback(move |address| {
                if let Some(window) = window_weak.upgrade() {
                    glib::spawn_future_local(async move {
                        window.forget_device(address).await;
                    });
                }
            });
        }
        
        // Connect scan button to scan action
        if let Some(scan_button) = imp.scan_button.borrow().as_ref() {
            let window_weak = self.downgrade();
            scan_button.connect_clicked(move |_| {
                if let Some(window) = window_weak.upgrade() {
                    glib::spawn_future_local(async move {
                        window.start_device_scan().await;
                    });
                }
            });
        }
        
        // Connect Bluetooth toggle switch
        if let Some(bluetooth_toggle) = imp.bluetooth_toggle.borrow().as_ref() {
            let window_weak = self.downgrade();
            bluetooth_toggle.connect_state_set(move |_, state| {
                if let Some(window) = window_weak.upgrade() {
                    glib::spawn_future_local(async move {
                        window.toggle_bluetooth(state).await;
                    });
                }
                glib::Propagation::Proceed
            });
        }
    }
    
    async fn initialize_bluetooth(&self) {
        match BluetoothManager::new().await {
            Ok(manager) => {
                let imp = self.imp();
                imp.bluetooth_manager.replace(Some(manager));
                log::info!("Bluetooth manager initialized successfully");
                
                // Check current Bluetooth adapter state and update toggle
                if let Some(bluetooth_manager) = imp.bluetooth_manager.borrow().as_ref() {
                    if let Some(adapter) = bluetooth_manager.get_default_adapter().await {
                        match adapter.is_powered().await {
                            Ok(powered) => {
                                if let Some(toggle) = imp.bluetooth_toggle.borrow().as_ref() {
                                    toggle.set_active(powered);
                                    log::info!("Set Bluetooth toggle to: {}", powered);
                                }
                                
                                // Only scan for devices if Bluetooth is powered on
                                if powered {
                                    self.start_device_scan().await;
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to check adapter power state: {}", e);
                                // Default to scanning anyway
                                self.start_device_scan().await;
                            }
                        }
                    } else {
                        log::warn!("No default Bluetooth adapter found");
                        if let Some(toggle) = imp.bluetooth_toggle.borrow().as_ref() {
                            toggle.set_active(false);
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to initialize Bluetooth manager: {}", e);
                self.show_error_message(&format!("Failed to initialize Bluetooth: {}", e));
                
                // Set toggle to off on initialization failure
                let imp = self.imp();
                if let Some(toggle) = imp.bluetooth_toggle.borrow().as_ref() {
                    toggle.set_active(false);
                }
            }
        }
    }
    
    async fn start_device_scan(&self) {
        let imp = self.imp();
        
        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
            // Start discovery
            if let Err(e) = manager.start_discovery().await {
                log::error!("Failed to start device discovery: {}", e);
                self.show_error_message(&format!("Failed to start scanning: {}", e));
                return;
            }
            
            log::info!("Started device discovery, scanning for devices...");
            
            // Get initial devices immediately
            match manager.get_devices().await {
                Ok(devices) => {
                    log::info!("Initial scan: Found {} devices", devices.len());
                    self.update_device_list(devices);
                }
                Err(e) => {
                    log::error!("Failed to get initial devices: {}", e);
                    self.show_error_message(&format!("Failed to get devices: {}", e));
                }
            }
            
            // Schedule a single delayed update instead of continuous polling
            let window_weak = self.downgrade();
            let source_id = glib::timeout_add_seconds_local(3, move || {
                if let Some(window) = window_weak.upgrade() {
                    glib::spawn_future_local(async move {
                        let imp = window.imp();
                        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
                            match manager.get_devices().await {
                                Ok(devices) => {
                                    log::info!("Auto-scan update: Found {} devices", devices.len());
                                    window.update_device_list(devices);
                                }
                                Err(e) => {
                                    log::error!("Failed to get devices in auto-scan: {}", e);
                                }
                            }
                        }
                    });
                    glib::ControlFlow::Continue
                } else {
                    glib::ControlFlow::Break
                }
            });
            
            // Store the source ID to be able to stop auto-scan later if needed
            imp.auto_scan_source_id.replace(Some(source_id));
            
            log::info!("Device discovery initiated with continuous auto-scan every 3 seconds");
        } else {
            log::warn!("Bluetooth manager not initialized");
            self.show_error_message("Bluetooth manager not initialized");
        }
    }
    
    fn update_device_list(&self, devices: Vec<crate::bluetooth::device::BluetoothDevice>) {
        log::debug!("Updating device list with {} devices", devices.len());
        let imp = self.imp();
        
        if let Some(device_list) = imp.device_list.borrow().as_ref() {
            log::debug!("Device list widget found, updating efficiently");
            device_list.update_devices_efficiently(devices);
            log::debug!("Device list update complete");
        } else {
            log::error!("Device list widget not found!");
        }
    }
    
    fn show_error_message(&self, message: &str) {
        // For now, just log the error. In a full implementation, 
        // you'd show a proper error dialog
        log::error!("Error: {}", message);
    }
    
    fn show_info_message(&self, message: &str) {
        // For now, just log the info. In a full implementation, 
        // you'd show a proper info notification
        log::info!("Info: {}", message);
    }
    
    async fn connect_device(&self, address: String) {
        log::info!("Connecting to device: {}", address);
        let imp = self.imp();
        
        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
            match manager.connect_device(&address).await {
                Ok(()) => {
                    log::info!("Successfully connected to device: {}", address);
                    self.show_info_message(&format!("Connected to {}", address));
                    // Refresh the device list to update connection status
                    self.refresh_device_list().await;
                }
                Err(e) => {
                    log::error!("Failed to connect to device {}: {}", address, e);
                    self.show_error_message(&format!("Failed to connect: {}", e));
                }
            }
        } else {
            log::warn!("Bluetooth manager not initialized");
            self.show_error_message("Bluetooth manager not initialized");
        }
    }
    
    async fn disconnect_device(&self, address: String) {
        log::info!("Disconnecting from device: {}", address);
        let imp = self.imp();
        
        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
            match manager.disconnect_device(&address).await {
                Ok(()) => {
                    log::info!("Successfully disconnected from device: {}", address);
                    self.show_info_message(&format!("Disconnected from {}", address));
                    // Refresh the device list to update connection status
                    self.refresh_device_list().await;
                }
                Err(e) => {
                    log::error!("Failed to disconnect from device {}: {}", address, e);
                    self.show_error_message(&format!("Failed to disconnect: {}", e));
                }
            }
        } else {
            log::warn!("Bluetooth manager not initialized");
            self.show_error_message("Bluetooth manager not initialized");
        }
    }
    
    async fn forget_device(&self, address: String) {
        log::info!("Forgetting device: {}", address);
        let imp = self.imp();
        
        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
            match manager.remove_device(&address).await {
                Ok(()) => {
                    log::info!("Successfully removed device: {}", address);
                    self.show_info_message(&format!("Forgotten {}", address));
                    // Refresh the device list to remove the device
                    self.refresh_device_list().await;
                }
                Err(e) => {
                    log::error!("Failed to remove device {}: {}", address, e);
                    self.show_error_message(&format!("Failed to forget device: {}", e));
                }
            }
        } else {
            log::warn!("Bluetooth manager not initialized");
            self.show_error_message("Bluetooth manager not initialized");
        }
    }
    
    async fn refresh_device_list(&self) {
        log::info!("Refreshing device list");
        let imp = self.imp();
        
        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
            match manager.get_devices().await {
                Ok(devices) => {
                    log::info!("Refreshed device list with {} devices", devices.len());
                    self.update_device_list(devices);
                }
                Err(e) => {
                    log::error!("Failed to refresh device list: {}", e);
                }
            }
        }
    }
    
    async fn toggle_bluetooth(&self, enabled: bool) {
        log::info!("Toggling Bluetooth: {}", if enabled { "ON" } else { "OFF" });
        let imp = self.imp();
        
        if let Some(manager) = imp.bluetooth_manager.borrow().as_ref() {
            match manager.set_adapter_powered(enabled).await {
                Ok(()) => {
                    log::info!("Successfully {} Bluetooth", if enabled { "enabled" } else { "disabled" });
                    if enabled {
                        self.show_info_message("Bluetooth enabled");
                        // Automatically scan for devices when Bluetooth is turned on
                        self.start_device_scan().await;
                    } else {
                        self.show_info_message("Bluetooth disabled");
                        // Clear device list when Bluetooth is turned off
                        self.update_device_list(vec![]);
                    }
                }
                Err(e) => {
                    log::error!("Failed to {} Bluetooth: {}", if enabled { "enable" } else { "disable" }, e);
                    self.show_error_message(&format!("Failed to {} Bluetooth: {}", if enabled { "enable" } else { "disable" }, e));
                    
                    // Revert the toggle state on error
                    if let Some(toggle) = imp.bluetooth_toggle.borrow().as_ref() {
                        toggle.set_active(!enabled);
                    }
                }
            }
        } else {
            log::warn!("Bluetooth manager not initialized");
            self.show_error_message("Bluetooth manager not initialized");
            
            // Revert the toggle state on error
            if let Some(toggle) = imp.bluetooth_toggle.borrow().as_ref() {
                toggle.set_active(!enabled);
            }
        }
    }
    
    pub fn stop_auto_scan(&self) {
        let imp = self.imp();
        if let Some(source_id) = imp.auto_scan_source_id.take() {
            source_id.remove();
            log::info!("Auto-scan stopped");
        }
    }
}
