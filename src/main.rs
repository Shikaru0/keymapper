use clap::Parser;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Lists all available evdev input devices.
    ListDevices,
    /// Monitors a specific evdev input device for key events.
    Monitor {
        /// Path to the evdev input device (e.g., /dev/input/event0).
        device: String,
    },
    /// Remaps keys on a specific evdev input device. If no device is specified, the input device specified in the configuration file will be used.
    Remap {
        /// Path to the evdev input device (e.g., /dev/input/event0). If omitted, the input device specified in the configuration file will be used.
        device: Option<String>,
    },
    /// Prints the version of the keymapper package.
    PrintVersion,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::ListDevices) => keymapper::list_devices(),
        Some(Commands::Monitor { device }) => keymapper::monitor_device(&device),
        Some(Commands::Remap { device }) => keymapper::run_remap(device.as_deref()),
        Some(Commands::PrintVersion) => keymapper::print_version(),
        None => keymapper::print_version(),
    }
}
