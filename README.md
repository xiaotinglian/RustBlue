# RustBlue: Rust GTK4 Bluetooth Manager

A modern Bluetooth management application written in Rust using GTK4, inspired by the original [Blueman](https://github.com/blueman-project/blueman) project. This application provides a native desktop experience for managing Bluetooth devices on Fedora Linux.

![alt text](image-1.png)
![alt text](image-1.png)
The application has been successfully implemented and is ready for use on Fedora Linux systems.
the binary should be able to run in any linux

## Quick Start

### Prerequisites

The following system packages are required (automatically installed via the "Install Dependencies" task):

```bash
sudo dnf install -y gtk4-devel libadwaita-devel dbus-devel bluez-libs-devel
```

### Building and Running

1. **Using VS Code Tasks** (Recommended):
   - Press `Ctrl+Shift+P` and run "Tasks: Run Task"
   - Select "Run RustBlue" to build and launch the application

2. **Using Command Line**:
   ```bash
   cargo build        # Build the project
   cargo run          # Build and run the application
   cargo check        # Check for errors without building
   ```

### Installation

```bash
cargo build --release
sudo cp target/release/rustblue /usr/local/bin/
```

or take the binary from release


### Dependencies

- **GTK4 0.8**: Modern GUI toolkit
- **libadwaita 0.6**: GNOME design language
- **bluer 0.17**: Bluetooth stack integration
- **tokio**: Async runtime
- **dbus**: System bus communication
- **anyhow**: Error handling
