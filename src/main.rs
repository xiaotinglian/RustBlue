mod bluetooth;
mod ui;

use std::env;

use anyhow::Result;
use gio::prelude::*;
use gtk::{prelude::*, Application};
use log::{debug, info};

use ui::window::RustBlueWindow;

const APP_ID: &str = "org.rustblue.Manager";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting RustBlue");

    // Initialize GTK
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    // Set up command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Run the application
    let exit_code = app.run_with_args(&args);
    
    info!("Application exited with code: {:?}", exit_code);
    std::process::exit(exit_code.into());
}

fn build_ui(app: &Application) {
    debug!("Building UI");
    
    // Create the main window
    let window = RustBlueWindow::new(app);
    
    // Present the window
    window.present();
}
