# OpenInfiniFactory

A small Bevy prototype for a 3D block puzzle factory game.

## Run

```bash
cargo run
```

## Web

Build WebGPU-only wasm with the project `webgpu` feature:

```bash
trunk build --release --features webgpu --dist dist/web
```

Serve the generated files over `http://localhost` or HTTPS. Browser WebGPU is
only exposed in secure contexts; opening `dist/web/index.html` directly from the
filesystem is not a valid WebGPU runtime environment.

## Packaging

macOS `.app` bundle:

```bash
scripts/package_macos_app.sh
```

Linux loose binary package:

```bash
scripts/package_linux.sh
```

Windows loose binary package:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1
```

Android APK package:

```bash
scripts/package_android.sh
```

Packaging outputs are written under `dist/`. The scripts support `CARGO`,
`CARGO_APK`, and `TARGET` environment overrides for local toolchain adapters and
cross-compilation. Assets are resolved through the platform adapter in
`src/shared/platform.rs`, so development runs, loose binaries, `.app` bundles,
and Android APK assets can use different layouts without hard-coded paths. Set
`OPEN_INFINIFACTORY_ASSET_DIR` to override the asset directory for desktop
builds.

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
