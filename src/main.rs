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
    /// Prints the version of the keymapper package.
    PrintVersion,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::ListDevices) => keymapper::list_devices(),
        Some(Commands::PrintVersion) => keymapper::print_version(),
        None => keymapper::print_version(),
    }
}
