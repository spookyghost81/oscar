# oscar

oscar is an OSC-based DAW controller app written in Rust with a Lua-scriptable UI layout system.

It uses:

- `winit` for the window/event loop
- `wgpu` + `egui-wgpu` + `epaint` for rendering
- `mlua` (Luau) for runtime UI scripting
- `rosc` for OSC messaging

## Current Status

- Desktop app renders a custom controller UI from Lua draw commands.
- UI script state receives live DAW/OSC state updates.
- Lua can send track control messages back out over OSC.
- Default layout script: `lua/basic.lua`.

## Project Layout

- `src/main.rs` - app entry point.
- `src/app.rs` - window lifecycle, input handling, render loop, script reload keybind.
- `src/daw_control.rs` - OSC message mapping, DAW state model, network thread.
- `src/ui_script.rs` - Lua runtime setup, Lua-to-Rust bridge, draw command collection.
- `lua/` - scriptable UI pages/components (`basic.lua`, widgets in `lua/lib/`).
- `fonts/` - font assets.

## Prerequisites

- Rust toolchain (stable)
- A DAW/OSC endpoint listening on `127.0.0.1:8656`

Notes:

- The OSC target and ports are currently hardcoded in `src/daw_control.rs`.
- On startup, the app sends initialization OSC messages (track count/bank select) and then listens for incoming OSC packets.
- Set up in Reaper in Options > Preferences > Control/OSC/Web > Add
    Control surface mode: OSC
    Device name: any
    Pattern Config: Default
    Mode: Local port
    Local listen port: 8656

## Run

From repo root:

```bash
cargo run
```

Useful commands:

```bash
cargo check
cargo test
```

## Controls

- `R` reloads the UI script (`basic`).
- `Esc` exits the app.
- Mouse input is forwarded to the loaded Lua page each frame.

## Lua Scripting Basics

The script receives a global `oscar` userdata with values/functions such as:

- `oscar.window_size`, `oscar.window_width`, `oscar.window_height`
- `oscar.daw` (current DAW state mirror)
- `oscar:track_control(track_index, control_type, value)`

Supported built-in track control types currently include:

- `volume`
- `pan`
- `mute`
- `solo`
- `record_arm`

Each page script is expected to return an object/table that supports the methods used by the runtime (`update`, `draw`, and mouse-state updates via `update_mouse_state`).