# Take a Break

A menu-bar / tray app that reminds you to take scheduled breaks (hydration, stretch, eat,
or custom types). It shows the time remaining until your next break in the menu bar, and
can interrupt you with either a native notification or a fullscreen takeover window with
an image, a message, and a live countdown.

Built with [Tauri v2](https://v2.tauri.app/) — a Rust backend (`src-tauri/`) plus a small
vanilla HTML/CSS/JS frontend (`src/`), with no bundler or JS framework.

## Requirements

- [Rust](https://rustup.rs/) (stable toolchain; MSRV is 1.90, see `src-tauri/Cargo.toml`)
- [Tauri CLI](https://v2.tauri.app/reference/cli/): `cargo install tauri-cli --version "^2"`
- **macOS**: Xcode Command Line Tools (`xcode-select --install`). Regenerating the tray/UI
  icons additionally needs the Swift toolchain, which ships with the Command Line Tools.
- **Linux**: system packages for WebKitGTK + the tray/appindicator libraries. On
  Debian/Ubuntu:

  ```sh
  # build-time
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

  # runtime (once packaged)
  sudo apt install libayatana-appindicator3-1 libwebkit2gtk-4.1-0
  ```

  Note: on Wayland there's no standard tray-icon protocol without a compositor extension
  (e.g. GNOME's AppIndicator extension), and always-on-top windows are more restricted
  than on X11 — the fullscreen break overlay may not reliably appear above another
  fullscreen app. X11 doesn't have either limitation.

## Quick start

```sh
make dev     # run the app with hot reload
```

This launches the tray icon and the (initially hidden) Settings window. Use the tray
menu's "Open Settings…" to bring it up.

## Common commands

Run `make help` to list these from the Makefile itself.

| Command          | What it does                                              |
|------------------|------------------------------------------------------------|
| `make dev`       | Run in development mode with hot reload                    |
| `make build`     | Build a release bundle for the current platform             |
| `make check`     | Type-check the Rust code without building                   |
| `make test`      | Run the Rust unit test suite                                 |
| `make fmt`       | Format Rust code                                              |
| `make fmt-check` | Check formatting without writing changes                     |
| `make lint`      | Run Clippy with warnings denied                               |
| `make ci`        | Run `fmt-check`, `lint`, and `test` together                  |
| `make icons`     | Regenerate the SF Symbols icon assets (macOS only, see below) |
| `make clean`     | Remove build artifacts                                        |

`make build` produces a platform-native bundle (`.app`/`.dmg` on macOS, `.deb`/`.AppImage`
on Linux) under `src-tauri/target/release/bundle/`. There's no cross-compilation set up —
build on the platform you're targeting.

## Project structure

```
src-tauri/               Rust backend (the actual Tauri app)
  src/
    models/              Break, Settings, AppData — the persisted data shapes
    persistence/          Load/save JSON (atomic writes) + image storage
    scheduler/            Pure trigger-decision logic (unit tested) + the tokio loop
    tray/                 Tray icon, menu, and the "Xm / Xh / Xd" label formatting
    call_detection/       "Cancel breaks when on a call" mic-in-use heuristic
    commands/             Tauri commands the frontend calls (CRUD, postpone/cancel, etc.)
    overlay/              Fullscreen break window lifecycle
  icons/                  Tray icon (raw RGBA) + app bundle icons
src/                     Frontend (plain HTML/CSS/JS, no build step)
  index.html, assets/settings.js   Settings window
  overlay.html, assets/overlay.js  Fullscreen break window
  assets/icons/           SF Symbols exported as PNG masks (see scripts/)
scripts/
  export-sf-symbols.swift  Renders real Apple SF Symbols to the PNGs used above
```

## Data storage

Settings and the break schedule are stored as JSON in the OS app-config directory (e.g.
`~/Library/Application Support/dev.pedropiedade.takeabreak/` on macOS), written
atomically. Runtime-only scheduler state (postponed/skipped-today bookkeeping) is kept in
a separate `state.json` alongside it, so a postpone/cancel survives an app restart.
Picked images are copied into an `images/` subfolder there under a generated filename —
the original file path is never referenced again.

## Regenerating icons

The pencil/trash/plus/xmark/chevron/photo icons used in the Settings UI are real Apple
SF Symbols, exported once as PNG alpha masks (not baked-in colors — the frontend tints
them via CSS `mask-image` + `currentColor`). To add or change one, edit the `symbols`
list in `scripts/export-sf-symbols.swift` and run:

```sh
make icons
```

This only works on macOS (SF Symbols are an AppKit API).

## Known limitations

- **Notification-style breaks only work from a real build, never from `cargo tauri dev`.**
  On macOS, notifications go through `UNUserNotificationCenter` (see `notify.rs`) rather
  than `tauri-plugin-notification`'s default backend, which relies on the deprecated
  `NSUserNotification` API and was found to silently do nothing — permission shows as
  granted, the API call reports success, but nothing ever displays. `UNUserNotificationCenter`
  fixes that, but it fundamentally requires the process to be a signed `.app` bundle; a raw
  `cargo tauri dev` binary can't use it at all. **`cargo tauri build` does not sign the
  bundle by default** — there's no `bundle.macOS.signingIdentity` configured in
  `tauri.conf.json`, so both debug and release builds come out unsigned unless you add a
  Developer ID there (or set `APPLE_CERTIFICATE`/`APPLE_CERTIFICATE_PASSWORD` for CI). To
  test notification-style breaks without a paid Developer ID, ad-hoc sign the bundle
  yourself after building — a plain `-` identity is sufficient for local testing (not for
  distribution):
  ```sh
  cargo tauri build --debug
  codesign --force --deep --sign - "src-tauri/target/debug/bundle/macos/Take a Break.app"
  open "src-tauri/target/debug/bundle/macos/Take a Break.app"
  ```
  Fullscreen breaks are unaffected by any of this — they're plain windows, not notifications.
- Call detection ("Cancel breaks when on a call") is a best-effort CoreAudio mic-in-use
  poll on macOS; it's a stub that always reports "not on a call" on Linux.
- The fullscreen overlay intentionally renders below the macOS menu bar strip rather than
  covering it, to avoid needing raw NSWindow-level workarounds.
- Linux tray/overlay behavior on Wayland is compositor-dependent (see Requirements above).
