# OpenInfiniFactory

A small Bevy prototype for a 3D block puzzle factory game.

## Run

```bash
cargo run
```

## Controls

- `WASD`: move camera
- `Space`: jump
- Double-tap `Space`: toggle flying
- `Space` / `Shift`: fly up / down while flying
- Move mouse: look around while captured
- Left click: break targeted block
- Right click: place selected block
- `1`-`9`: select hotbar slot
- `R`: rotate placement direction
- `E` or `I`: open/close inventory and release mouse
- `F5`: save
- `F9`: load
- `Esc`: pause/release mouse, or return to game
- Pause menu: resume, adjust FOV, or quit
- `/`: toggle debug overlay with FPS and player collision box
- Pause menu can switch Edit/Play build modes
- Top-right controls handle turn playback, speed, and rollback in Play mode

Saves are written to `saves/world.ron`.

## Font Notice

`assets/fonts/PingFangSC-Regular.ttf` is exported from the macOS system PingFang
font collection for Chinese UI rendering. PingFang is an Apple system font; check
Apple's font license before redistributing this asset outside local development.
