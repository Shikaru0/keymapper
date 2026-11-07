# keymapper

A lightweight Linux/Wayland key remapper written in Rust, using [evdev](https://docs.rs/evdev/latest/evdev/) for input handling and `uinput` for virtual device creation.

[![CI](https://github.com/Shikaru0/keymapper/actions/workflows/rust.yml/badge.svg)](https://github.com/Shikaru0/keymapper/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-CE412B?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/Wayland-Supported-FFCC00?logo=wayland&logoColor=black)](https://wayland.freedesktop.org/)

## Building

*_Requires Rust_*

```bash
git clone https://github.com/Shikaru0/keymapper.git
cd keymapper
cargo build --release
```

## Usage

```bash
keymapper list-devices              # list evdev input devices
keymapper monitor /dev/input/event0 # stream key events to stdout
keymapper remap                     # start the daemon (reads config.toml)
keymapper remap /dev/input/event0   # start with an explicit device
```

## Config

Auto-generated at `~/.config/keymapper/config.toml` on first run:

```toml
# Configuration for keymapper.

# Path to the evdev input device (e.g., /dev/input/event0).
input_device = "/dev/input/event0"

# Remap table for keymapper.
[[remap]]
from = "KEY_A"
to = "KEY_B"

[[remap]]
from = "KEY_CAPSLOCK"
to = "KEY_ESC"

[[remap]]
from = "KEY_F1"
to = ["KEY_LEFTCTRL", "KEY_T"]
```

`to` can be a single key name or a list of key names for combo triggers. Key names are the `KEY_*` constants from `/usr/include/linux/input-event-codes.h`.

Editing config.toml while the daemon runs hot-swaps the remap table, so no restart is needed.

## Permissions

```bash
sudo usermod -aG input $USER
sudo usermod -aG uinput $USER
```

Log out and back in for the changes to take effect.

## How does it work?

The daemon grabs the physical input device exclusively, creates a `uinput` virtual keyboard, and forwards every event through it — translating keys that match a remap rule before emitting. This works under both X11 and Wayland.
