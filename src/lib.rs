pub mod config;

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

/// Remaps keys on a specific evdev input device.
pub fn run_remap(path: Option<&str>) {
    let config = config::load_config();
    let remap = config.remap_table();
    let path = match path {
        Some(p) => p.to_owned(),
        None => config.input_device.clone(),
    };

    let mut device = match evdev::Device::open(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open device {}: {}", path, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = device.grab() {
        eprintln!("Failed to grab device {}: {}", path, e);
        std::process::exit(1);
    }

    let mut keys: evdev::AttributeSet<evdev::KeyCode> = match device.supported_keys() {
        Some(supported) => supported.iter().collect(),
        None => evdev::AttributeSet::new(),
    };
    for (_, target) in &remap {
        keys.insert(*target);
    }

    let mut virtual_device = match evdev::uinput::VirtualDevice::builder()
        .expect("Failed to create virtual device builder")
        .name("keymapper-virtual-device")
        .with_keys(&keys)
        .expect("Failed to set virtual device keys")
        .build()
    {
        Ok(vd) => vd,
        Err(e) => {
            eprintln!("Failed to create virtual device: {}", e);
            std::process::exit(1);
        }
    };

    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Error fetching events: {}", e);
                std::process::exit(1);
            }
        };
        let mut out: Vec<evdev::InputEvent> = Vec::new();
        for event in events {
            let event = if let evdev::EventSummary::Key(_, keycode, value) = event.destructure() {
                let remapped = remap
                    .iter()
                    .find(|(src, _)| *src == keycode)
                    .map(|(_, dst)| *dst)
                    .unwrap_or(keycode);
                evdev::InputEvent::new(evdev::EventType::KEY.0, remapped.code(), value)
            } else {
                event
            };
            out.push(event);
        }
        if !out.is_empty()
            && let Err(e) = virtual_device.emit(&out)
        {
            eprintln!("Error emitting events: {}", e);
        }
    }
}
