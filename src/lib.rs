pub mod config;

use std::{
    sync::{Arc, Mutex},
    thread,
};

use notify::Watcher;

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

fn spawn_config_watcher(remap: Arc<Mutex<Vec<(evdev::KeyCode, evdev::KeyCode)>>>) {
    let path = config::config_path();
    let parent = path.parent().expect("Failed to get config directory");

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).expect("Failed to create file watcher");
    watcher
        .watch(parent, notify::RecursiveMode::NonRecursive)
        .expect("Failed to watch config directory");

    thread::spawn(move || {
        let _watcher = watcher; // Keep the watcher alive in this thread

        for res in rx.iter() {
            match res {
                Ok(event) => {
                    if event.paths.contains(&path) {
                        println!("Configuration file changed, reloading...");
                        let config = config::try_load_config();
                        if let Some(cfg) = config {
                            let new_remap = cfg.remap_table();
                            let mut remap_lock = remap.lock().unwrap();
                            *remap_lock = new_remap;
                            println!("Remap table updated.");
                        } else {
                            eprintln!("Failed to reload configuration.");
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }
    });
}

/// Remaps keys on a specific evdev input device.
pub fn run_remap(path: Option<&str>) {
    let config = config::load_config();
    let remap_vec = config.remap_table();
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
    for (_, target) in &remap_vec {
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

    let remap = Arc::new(Mutex::new(remap_vec));
    spawn_config_watcher(Arc::clone(&remap));

    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Error fetching events: {}", e);
                std::process::exit(1);
            }
        };
        let remap_snapshot = remap.lock().unwrap().clone();
        let mut out: Vec<evdev::InputEvent> = Vec::new();
        for event in events {
            let event = if let evdev::EventSummary::Key(_, keycode, value) = event.destructure() {
                let remapped = remap_snapshot
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
