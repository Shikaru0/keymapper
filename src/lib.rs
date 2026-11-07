/// Package version, defined in Cargo.toml, is embedded into the binary at compile time using the `env!` macro.
/// This allows the program to access its own version information at runtime.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prints the version of the keymapper package to the console.
pub fn print_version() {
    println!("Keymapper version: {}", VERSION);
}
