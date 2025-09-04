use gtk::{
    glib, prelude::*, subclass::prelude::*, Box as GtkBox, Button, Label, ListBox, Orientation,
    SelectionMode, Widget, ScrolledWindow,
};

use crate::bluetooth::device::BluetoothDevice;

glib::wrapper! {
    pub struct DeviceListView(ObjectSubclass<imp::DeviceListView>)
        @extends GtkBox, Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl DeviceListView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
    
    pub fn set_connect_callback<F>(&self, callback: F) 
    where
        F: Fn(String) + 'static,
    {
        let imp = self.imp();
        *imp.connect_callback.borrow_mut() = Some(Box::new(callback));
    }
    
    pub fn set_disconnect_callback<F>(&self, callback: F) 
    where
        F: Fn(String) + 'static,
    {
        let imp = self.imp();
        *imp.disconnect_callback.borrow_mut() = Some(Box::new(callback));
    }
    
    pub fn set_forget_callback<F>(&self, callback: F) 
    where
        F: Fn(String) + 'static,
    {
        let imp = self.imp();
        *imp.forget_callback.borrow_mut() = Some(Box::new(callback));
    }
    
    pub fn add_device(&self, device: &BluetoothDevice) {
        log::info!("Adding device to UI: {} ({})", device.name, device.address);
        let imp = self.imp();
        
        // Create a compact device row with horizontal layout
        let device_row = GtkBox::new(Orientation::Horizontal, 8);
        device_row.set_margin_top(6);
        device_row.set_margin_bottom(6);
        device_row.set_margin_start(12);
        device_row.set_margin_end(12);
        device_row.set_hexpand(true);
        
        // Left side - device info
        let info_box = GtkBox::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);
        
        let name_label = Label::new(Some(&device.name));
        name_label.set_halign(gtk::Align::Start);
        name_label.set_markup(&format!("<b>{}</b>", device.name));
        name_label.set_hexpand(true);
        
        let address_label = Label::new(Some(&device.address));
        address_label.set_halign(gtk::Align::Start);
        address_label.set_markup(&format!("<small>{}</small>", device.address));
        address_label.add_css_class("dim-label");
        address_label.set_hexpand(true);
        
        let status_label = Label::new(Some(if device.connected { "Connected" } else { "Disconnected" }));
        status_label.set_halign(gtk::Align::Start);
        status_label.set_markup(&format!("<small><i>{}</i></small>", 
            if device.connected { "Connected" } else { "Disconnected" }));
        status_label.add_css_class(if device.connected { "success" } else { "warning" });
        status_label.set_hexpand(true);
        
        info_box.append(&name_label);
        info_box.append(&address_label);
        info_box.append(&status_label);
        
        // Right side - action buttons (compact spacing)
        let button_box = GtkBox::new(Orientation::Horizontal, 4);
        button_box.set_halign(gtk::Align::End);
        button_box.set_valign(gtk::Align::Center);
        
        // Connect/Disconnect button
        let connection_button = if device.connected {
            Button::with_label("Disconnect")
        } else {
            Button::with_label("Connect")
        };
        
        if device.connected {
            connection_button.add_css_class("destructive-action");
        } else {
            connection_button.add_css_class("suggested-action");
        }
        
        // Clone device address for button callbacks
        let device_address = device.address.clone();
        let device_connected = device.connected;
        
        connection_button.connect_clicked(glib::clone!(@weak self as device_list => move |_| {
            log::info!("Connection button clicked for device: {}", device_address);
            if device_connected {
                log::info!("Attempting to disconnect device: {}", device_address);
                let imp = device_list.imp();
                let callback = imp.disconnect_callback.borrow();
                if let Some(ref cb) = *callback {
                    cb(device_address.clone());
                }
            } else {
                log::info!("Attempting to connect device: {}", device_address);
                let imp = device_list.imp();
                let callback = imp.connect_callback.borrow();
                if let Some(ref cb) = *callback {
                    cb(device_address.clone());
                }
            }
        }));
        
        // Forget button (only show for paired devices)
        let forget_button = Button::with_label("Forget");
        forget_button.add_css_class("destructive-action");
        
        let device_address_forget = device.address.clone();
        forget_button.connect_clicked(glib::clone!(@weak self as device_list => move |_| {
            log::info!("Forget button clicked for device: {}", device_address_forget);
            let imp = device_list.imp();
            let callback = imp.forget_callback.borrow();
            if let Some(ref cb) = *callback {
                cb(device_address_forget.clone());
            }
        }));
        
        button_box.append(&connection_button);
        if device.paired {
            button_box.append(&forget_button);
        }
        
        device_row.append(&info_box);
        device_row.append(&button_box);
        
        let list_box = imp.list_box.borrow();
        list_box.append(&device_row);
        log::info!("Device added to list box with action buttons");
    }
    
    pub fn clear_devices(&self) {
        log::debug!("Clearing all devices from UI");
        let imp = self.imp();
        let list_box = imp.list_box.borrow();
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        log::debug!("All devices cleared");
    }
    
    pub fn update_devices_efficiently(&self, devices: Vec<BluetoothDevice>) {
        log::debug!("Efficiently updating device list with {} devices", devices.len());
        let imp = self.imp();
        
        let list_box = imp.list_box.borrow();
        
        // Clear existing devices
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        
        // Sort devices: connected devices first, then by name
        let mut sorted_devices = devices;
        sorted_devices.sort_by(|a, b| {
            match (a.connected, b.connected) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        // Add devices efficiently
        for device in sorted_devices {
            self.add_device_internal(&device);
        }
        
        log::debug!("Efficient device list update complete");
    }
    
    fn add_device_internal(&self, device: &BluetoothDevice) {
        let imp = self.imp();
        
        // Create a compact device row with horizontal layout (optimized)
        let device_row = GtkBox::new(Orientation::Horizontal, 8);
        device_row.set_margin_top(6); // Reduced margin for compact view
        device_row.set_margin_bottom(6);
        device_row.set_margin_start(12);
        device_row.set_margin_end(12);
        device_row.set_hexpand(true);
        
        // Left side - device info
        let info_box = GtkBox::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);
        
        let name_label = Label::new(Some(&device.name));
        name_label.set_halign(gtk::Align::Start);
        name_label.set_markup(&format!("<b>{}</b>", device.name));
        name_label.set_hexpand(true);
        
        let address_label = Label::new(Some(&device.address));
        address_label.set_halign(gtk::Align::Start);
        address_label.set_markup(&format!("<small>{}</small>", device.address));
        address_label.add_css_class("dim-label");
        address_label.set_hexpand(true);
        
        let status_label = Label::new(Some(if device.connected { "Connected" } else { "Disconnected" }));
        status_label.set_halign(gtk::Align::Start);
        status_label.set_markup(&format!("<small><i>{}</i></small>", 
            if device.connected { "Connected" } else { "Disconnected" }));
        status_label.add_css_class(if device.connected { "success" } else { "warning" });
        status_label.set_hexpand(true);
        
        info_box.append(&name_label);
        info_box.append(&address_label);
        info_box.append(&status_label);
        
        // Right side - action buttons (compact spacing)
        let button_box = GtkBox::new(Orientation::Horizontal, 4);
        button_box.set_halign(gtk::Align::End);
        button_box.set_valign(gtk::Align::Center);
        
        // Connect/Disconnect button
        let connection_button = if device.connected {
            Button::with_label("Disconnect")
        } else {
            Button::with_label("Connect")
        };
        
        if device.connected {
            connection_button.add_css_class("destructive-action");
        } else {
            connection_button.add_css_class("suggested-action");
        }
        
        // Clone device address for button callbacks
        let device_address = device.address.clone();
        let device_connected = device.connected;
        
        connection_button.connect_clicked(glib::clone!(@weak self as device_list => move |_| {
            if device_connected {
                let callback = device_list.imp().disconnect_callback.borrow();
                if let Some(ref cb) = *callback {
                    cb(device_address.clone());
                }
            } else {
                let callback = device_list.imp().connect_callback.borrow();
                if let Some(ref cb) = *callback {
                    cb(device_address.clone());
                }
            }
        }));
        
        // Forget button (only show for paired devices)
        let forget_button = Button::with_label("Forget");
        forget_button.add_css_class("destructive-action");
        
        let device_address_forget = device.address.clone();
        forget_button.connect_clicked(glib::clone!(@weak self as device_list => move |_| {
            let callback = device_list.imp().forget_callback.borrow();
            if let Some(ref cb) = *callback {
                cb(device_address_forget.clone());
            }
        }));
        
        button_box.append(&connection_button);
        if device.paired {
            button_box.append(&forget_button);
        }
        
        device_row.append(&info_box);
        device_row.append(&button_box);
        
        let list_box = imp.list_box.borrow();
        list_box.append(&device_row);
    }
}

mod imp {
    use super::*;
    use std::cell::RefCell;

    type CallbackFn = Box<dyn Fn(String)>;

    pub struct DeviceListView {
        pub list_box: RefCell<ListBox>,
        pub connect_callback: RefCell<Option<CallbackFn>>,
        pub disconnect_callback: RefCell<Option<CallbackFn>>,
        pub forget_callback: RefCell<Option<CallbackFn>>,
    }

    impl Default for DeviceListView {
        fn default() -> Self {
            Self {
                list_box: RefCell::new(ListBox::new()),
                connect_callback: RefCell::new(None),
                disconnect_callback: RefCell::new(None),
                forget_callback: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DeviceListView {
        const NAME: &'static str = "DeviceListView";
        type Type = super::DeviceListView;
        type ParentType = GtkBox;
    }

    impl ObjectImpl for DeviceListView {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.set_orientation(Orientation::Vertical);
            obj.set_spacing(8);
            obj.set_hexpand(true);
            obj.set_vexpand(true);
            obj.add_css_class("view");
            
            // Create header with background
            let header = Label::new(Some("Bluetooth Devices"));
            header.set_markup("<big><b>Bluetooth Devices</b></big>");
            header.set_halign(gtk::Align::Start);
            header.set_margin_top(16);
            header.set_margin_bottom(8);
            header.set_margin_start(16);
            header.set_margin_end(16);
            obj.append(&header);
            
            // Create list box with visible styling
            let list_box = ListBox::new();
            list_box.set_selection_mode(SelectionMode::Single);
            list_box.add_css_class("navigation-sidebar");
            list_box.set_hexpand(true);
            list_box.set_vexpand(true);
            
            // Create scrolled window to contain the list box
            let scrolled_window = ScrolledWindow::new();
            scrolled_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
            scrolled_window.set_hexpand(true);
            scrolled_window.set_vexpand(true);
            scrolled_window.set_margin_start(16);
            scrolled_window.set_margin_end(16);
            scrolled_window.set_margin_bottom(16);
            scrolled_window.set_min_content_height(300);
            scrolled_window.set_max_content_height(600);
            
            // Add list box to scrolled window
            scrolled_window.set_child(Some(&list_box));
            
            obj.append(&scrolled_window);
            
            // Store the list_box reference
            *self.list_box.borrow_mut() = list_box;
            
            log::info!("DeviceListView constructed with proper sizing");
        }
    }

    impl WidgetImpl for DeviceListView {}
    impl BoxImpl for DeviceListView {}
}
