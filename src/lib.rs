/// Package version, defined in Cargo.toml, is embedded into the binary at compile time using the `env!` macro.
/// This allows the program to access its own version information at runtime.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prints the version of the keymapper package to the console.
pub fn print_version() {
    println!("Keymapper version: {}", VERSION);
}

/// Lists all available evdev input devices.
pub fn list_devices() {
    let devices: Vec<_> = evdev::enumerate().collect();
    for (i, (path, device)) in devices.iter().enumerate() {
        println!(
            "{}: {} ({})",
            i,
            device.name().unwrap_or("Unknown"),
            path.display()
        );
    }
}

/// Monitors a specific evdev input device for key events.
pub fn monitor_device(path: &str) {
    let mut device = evdev::Device::open(path).expect("Failed to open device");
    println!("Monitoring device: {}", device.name().unwrap_or("Unknown"));

    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Error fetching events: {}", e);
                std::process::exit(1);
            }
        };
        for event in events {
            if let evdev::EventSummary::Key(_, keycode, value) = event.destructure() {
                let state = match value {
                    0 => "released",
                    1 => "pressed",
                    2 => "held",
                    _ => "unknown",
                };
                println!("Key {:?} was {}", keycode, state);
            }
        }
    }
}
