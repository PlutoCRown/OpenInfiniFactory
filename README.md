# OpenInfiniFactory

A small Bevy prototype for a 3D block puzzle factory game.

## Architecture

Two independent Bevy runtimes share the same simulation logic:

| Runtime | Entry | Bevy scope |
|---------|-------|------------|
| Game client | `cargo run` | Window, UI, 3D scene, full plugins |
| Headless sim | `cargo run --bin oif-debug-http` | ECS resources only (`SimCorePlugin`), no window or rendering |

HTTP debug can attach to either runtime (`--debug-http` in the game, or the headless binary). E2E tests drive the headless ECS App.

See `docs/report/architecture.md` for layer breakdown.

## Run (game client)

```bash
cargo run
```

With embedded debug HTTP (default port 8765):

```bash
cargo run -- --debug-http
cargo run -- --debug-http=9000
```

Load a save on startup:

```bash
cargo run -- --load-save=world
```

## Run (headless sim + HTTP)

Build and start the headless ECS server (no window):

```bash
cargo build --bin oif-debug-http
cargo run --bin oif-debug-http
```

Options:

```bash
cargo run --bin oif-debug-http -- --debug-http=8765
cargo run --bin oif-debug-http -- --load-save=world
cargo run --bin oif-debug-http -- --load-fixture=blocks/Conveyor.json
```

## E2E tests

```bash
cargo build --bin oif-debug-http
cd e2e && bun run generate-fixtures && bun test
```

Details: `e2e/README.md`.

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
