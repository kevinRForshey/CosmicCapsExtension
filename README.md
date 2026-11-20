# CosmicCapsExtension

This is an extension for the Cosmic Desktop that indicates if your capslock button is on or not. Very simple and built with my own custom Spec Driven Development end to end development process.

A [COSMIC desktop](https://system76.com/cosmic) panel applet that shows the
text **"CapsLockOn"** in the panel whenever Caps Lock is active, and hides
it the instant Caps Lock is turned back off. Font, text color, and text
size are user-configurable from the applet's own settings popup; panel
placement (Start/Center/End) is handled by COSMIC's own Settings → Panel
UI, like any other applet.

Built end-to-end with a custom Spec-Driven Development (SDD) process —
`spec.md` → `tech-spec.md` → `tests.md` → `stories.md` — all under
[`docs/sdd/capslock-indicator-panel-extension/`](docs/sdd/capslock-indicator-panel-extension/).

## Features

- Detects Caps Lock state via the Linux sysfs keyboard-LED class
  (`/sys/class/leds/input*::capslock/brightness`), polled every 100ms on a
  dedicated background thread — no Wayland keyboard-focus trickery needed.
- Renders "CapsLockOn" in the panel only while Caps Lock is on; the widget
  takes up no space while it's off.
- Settings popup lets you pick:
  - **Font family**, from a dropdown of fonts actually installed on the
    system (enumerated via `fontdb`).
  - **Text color**, via a hex (`#RRGGBB`) entry with a live swatch.
  - **Text size**, via a slider (8–32px).
- Settings persist across restarts/logout via `cosmic-config`.
- Panel zone placement is provided for free by registering as a standard
  `cosmic-panel` applet — drag it between Start/Center/End in COSMIC
  Settings → Panel like any other applet.

## Project layout

```
.
├── src/
│   ├── caps_detect.rs    # pure sysfs LED detection logic (unit tested)
│   ├── settings.rs       # pure IndicatorSettings + render/serialize logic (unit tested)
│   ├── desktop_entry.rs  # pure .desktop-entry structural validation (unit tested)
│   ├── config.rs         # cosmic-config persistence, wraps settings.rs
│   ├── app.rs            # the real cosmic::Application / iced GUI
│   ├── lib.rs
│   └── main.rs
├── data/
│   └── com.example.CosmicAppletCapsLock.desktop
├── docs/sdd/              # the spec/tech-spec/tests/stories this was built from
└── scripts/
    └── install.sh         # local build + install script
```

The `caps_detect`, `settings`, and `desktop_entry` modules are pure,
dependency-free Rust with full unit test coverage (16 tests), written and
locked in before any GUI code existed. `app.rs`/`config.rs` are a thin
`libcosmic`/`cosmic-config` wrapper around them — they call into
`settings::render_label_style()` rather than re-implementing styling
logic.

## Building

Requires a Rust toolchain and the system packages `libcosmic` itself
needs to build (Wayland/EGL dev headers, same as any other
`cosmic-applet-*`). Dependencies (`libcosmic`, `cosmic-config`) are
fetched from `https://github.com/pop-os/libcosmic` via git, since they
aren't published to crates.io.

```sh
cargo build --release
```

Run the existing pure-logic test suite (no COSMIC session required):

```sh
cargo test
```

## Installing locally

```sh
./scripts/install.sh
```

This builds a release binary, installs it to `~/.local/bin/`, and
installs the `.desktop` entry to `~/.local/share/applications/`. Make
sure `~/.local/bin` is on your `PATH` — `cosmic-panel` launches applets
by the bare binary name on `PATH`, following the same convention as the
official `cosmic-applet-*` packages (the app-id appears only in the
`.desktop` file's `Icon=`/filename, not in `Exec=`).

After installing, add the applet from **COSMIC Settings → Desktop →
Panel → (Start/Center/End) → Add applet → CapsLock Indicator**.

Remove it with:

```sh
./scripts/install.sh --uninstall
```

## Known caveats

- **Hidden widget vs. settings access**: per the tech spec, the panel
  widget is zero-size while Caps Lock is off, so there's currently no
  visible click target to open the settings popup until Caps Lock is on.
  This matches the locked spec wording literally; worth revisiting if a
  persistent (if minimal) click target is wanted.
- **Font enumeration dependency**: the tech spec calls for
  `cosmic_text::fontdb`; this implementation depends on the leaner
  `fontdb` crate directly instead (the same underlying font database
  `cosmic_text` re-exports), to avoid pulling in `cosmic_text`'s much
  heavier default dependencies (shaping/rasterization libraries this
  applet never needs) for font-name enumeration alone.
- **No custom icon**: the `.desktop` file references an `Icon=` name with
  no actual icon file shipped yet, so it'll fall back to a generic icon
  in panel-settings UI until one is added.
- **AC-7/AC-9 (drag-to-reposition, zone persistence across restart)**
  are structural guarantees of being a well-registered `cosmic-panel`
  applet (TS-006) rather than something this code implements directly,
  and still need manual verification in a running COSMIC session.
