---
title: CapsLock Indicator Panel Extension — Technical Spec
date_created: 2026-11-20
status: Draft
---

# CapsLock Indicator Panel Extension — Technical Spec

## Design overview
Build this as a standard COSMIC panel applet: a Rust crate on `libcosmic`'s
`cosmic::applet` framework (the same foundation as `cosmic-applet-battery`,
`cosmic-applet-time`, etc.), registered with `cosmic-panel` via a normal
`.desktop` entry/app-id. Caps Lock state is read from the Linux sysfs
keyboard-LED class (`/sys/class/leds/input*::capslock/brightness`) rather
than via Wayland input protocols — this is the same mechanism widely used
by status-bar Caps Lock widgets (waybar/polybar), is compositor-agnostic,
needs no keyboard focus or privileged protocol extension, and is far
simpler than tracking `xkbcommon` modifier-lock state from a surface that
never receives keyboard focus. Display settings (font/color/size) are the
applet's own state, persisted via `cosmic-config`. Panel placement
(left/center/right) is deliberately *not* reimplemented by this applet —
it is obtained for free by behaving as a standard, well-registered
cosmic-panel plugin, which COSMIC's own Settings → Panel UI already lets
the user drag between zones for every applet.

## Technical design items
- **TS-001** [REQ-001, CON-001]: Project structure follows the standard
  cosmic-applet convention: a Rust crate implementing `cosmic::Application`
  via `cosmic::applet`, built as a single binary, installed with a
  `<app-id>.desktop` entry (e.g. `com.example.CosmicAppletCapsLock`) so
  `cosmic-panel` can launch it and COSMIC Settings → Panel can list/move
  it — this is what makes the applet a real panel applet rather than a
  standalone app or tray icon.
- **TS-002** [REQ-001, REQ-002]: Caps Lock state is detected via the Linux
  sysfs keyboard-LED class device(s) matching glob
  `/sys/class/leds/input*::capslock/brightness`. All matching nodes are
  read; state is considered "on" if *any* matched node reports a non-zero
  value — this gives one system-wide boolean state regardless of how many
  physical keyboards are attached, matching `spec.md`'s stated assumption.
- **TS-003** [REQ-003]: A dedicated background thread polls the brightness
  value(s) from TS-002 every 100ms and sends a state-change event over a
  channel into the applet's `cosmic::iced::Subscription`, which drives
  `update()`/`view()` to re-render immediately — no panel/applet restart
  is ever required for a state change to show up.
- **TS-008** [REQ-003]: If the running system has *no*
  `/sys/class/leds/input*::capslock` node at all (e.g. an unsupported
  input driver, a container/sandbox, or a keyboard driver that doesn't
  expose this LED class), the applet stays permanently hidden in the
  panel (no visible error/disabled state) and logs a single warning to
  stderr/the systemd journal on startup. No second, fallback Caps Lock
  detection mechanism is implemented — chosen over a visible error state
  or a fallback detector to keep the applet's surface area small, since
  this sysfs LED class is near-universal on real Linux keyboards and this
  is expected to be a rare edge case rather than a common one.
- **TS-004** [REQ-001, REQ-002]: When the polled state (TS-002/TS-003) is
  "on," the applet's panel widget renders a text label reading
  "CapsLockOn" using the current `IndicatorSettings` (TS-005). When "off,"
  the applet renders no content (a zero-size widget), so it visually
  disappears from the panel row rather than showing a placeholder/dimmed
  state.
- **TS-005** [REQ-004, REQ-005, REQ-006, REQ-008]: Display settings are
  modeled as `IndicatorSettings { font_family: String, text_color: String
  (hex, e.g. "#RRGGBB"), text_size: f32 }`, deriving `cosmic_config`'s
  `CosmicConfigEntry`, stored under this applet's own `cosmic-config`
  app-id (`~/.config/cosmic/<app-id>/v1/`). Settings are loaded once on
  startup and written back to disk on every change, which is what makes
  REQ-008 persistence (font/color/size portion) hold with no extra
  bespoke I/O code.
- **TS-006** [REQ-007]: Panel-zone placement (left/center/right) is
  provided entirely by COSMIC's existing panel-configuration mechanism,
  not by a setting inside this applet. Because TS-001 registers the
  applet as a standard cosmic-panel plugin, it already appears as a
  draggable entry in COSMIC Settings → Panel, where the user moves it
  between the Start/Center/End columns exactly as with every other
  applet; `cosmic-panel`'s own config (outside this applet's control)
  is what persists that choice (covers REQ-008's placement portion).
- **TS-007** [REQ-004, REQ-005, REQ-006, REQ-008]: A settings popup
  (opened via the standard applet click/popup mechanism used by other
  cosmic-applet-* projects) hosts three controls: (a) a font-family
  dropdown populated by enumerating installed fonts through
  `cosmic_text::fontdb::Database::load_system_fonts()` — the font
  database `libcosmic`/`iced`'s own text stack already loads, so no new
  dependency is needed; (b) a hex-code text entry with a live color swatch
  preview; (c) a numeric size control (slider or spin box) bounded to
  8–32px, chosen to stay legible without overflowing the panel's row
  height. Each control's change handler updates in-memory
  `IndicatorSettings` and immediately persists it via TS-005, so changes
  apply without a logout or panel restart.

## Acceptance criteria
- **AC-1** [REQ-001, TS-002, TS-003, TS-004]: Given Caps Lock is off and
  the applet is running, when the sysfs capslock LED brightness becomes
  non-zero, then the panel shows "CapsLockOn" within one polling interval
  (~100ms) with no further user action.
- **AC-2** [REQ-002, TS-002, TS-003, TS-004]: Given Caps Lock is on and
  "CapsLockOn" is visible, when the sysfs capslock LED brightness returns
  to zero, then the text is removed from the panel within one polling
  interval, with no placeholder or dimmed element left in its place.
- **AC-3** [REQ-003, TS-003]: Given the applet is running and the panel is
  not restarted, when Caps Lock is toggled on and off repeatedly, then the
  panel display tracks every toggle without the user restarting the
  panel, logging out, or otherwise intervening.
- **AC-4** [REQ-004, TS-007]: Given the applet's settings popup is open,
  when the user selects a different font family from the dropdown, then
  the visible "CapsLockOn" text immediately re-renders in that font.
- **AC-5** [REQ-005, TS-007]: Given the applet's settings popup is open,
  when the user enters a new hex color value, then the visible
  "CapsLockOn" text immediately re-renders in that color.
- **AC-6** [REQ-006, TS-007]: Given the applet's settings popup is open,
  when the user changes the text-size control, then the visible
  "CapsLockOn" text immediately re-renders at that size.
- **AC-7** [REQ-007, TS-001, TS-006]: Given the applet is registered as a
  standard cosmic-panel plugin, when the user opens COSMIC Settings →
  Panel and drags the applet's entry into the Start, Center, or End
  column, then `cosmic-panel` positions the applet in that chosen zone.
- **AC-8** [REQ-008, TS-005]: Given the user has changed font, color, or
  text size in the settings popup, when the applet or panel restarts (or
  the user logs back in), then "CapsLockOn" renders using the same font,
  color, and size as before the restart.
- **AC-9** [REQ-007, REQ-008, TS-006]: Given the user has moved the
  applet to a given panel zone via COSMIC Settings → Panel, when the
  panel restarts or the user logs back in, then the applet still launches
  in that same zone (persistence here is owned by `cosmic-panel`'s own
  configuration, per TS-006).
- **AC-10** [REQ-003, TS-008]: Given the running system has no
  `/sys/class/leds/input*::capslock` node, when the applet starts, then
  it logs exactly one warning to stderr/the journal and renders no
  visible content or error state in the panel.

## Non-functional notes
- CON-001 (must run as a `cosmic-panel` applet) is realized structurally
  by TS-001 — the applet is a registered plugin binary, not a standalone
  app or tray icon.
- CON-002 (fonts limited to what's installed) is realized by TS-007's use
  of `cosmic_text::fontdb`'s system font enumeration — the dropdown can
  only ever list fonts the system has actually loaded, so no bundled or
  remote font can be selected.
