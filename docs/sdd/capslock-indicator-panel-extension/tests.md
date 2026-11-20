# CapsLock Indicator Panel Extension — Tests (Definition of Done)

Definition of done: every test below passes.

Test crate: `capsIndicator/` (Rust, `cargo test`, std-only — no test
framework introduced beyond Rust's built-in `#[test]`). Pure
detection/settings/desktop-entry logic lives in
`src/caps_detect.rs`, `src/settings.rs`, `src/desktop_entry.rs`; each file
currently holds only its `#[cfg(test)] mod tests` block — no production
code exists yet, so every test below fails to *compile* (cannot find
function/type in `super`), which is this crate's red state. Confirmed via
`cargo test` on 2026-11-20 (47 compile errors across the three files).

Note on AC-7/AC-9: per tech-spec.md TS-006, panel-zone placement itself is
owned by `cosmic-panel`'s own configuration, not this applet's code. The
tests below cover only the structural precondition this crate is
responsible for (a well-formed, stably app-id'd `.desktop` entry). The
actual end-to-end zone-drag behavior in COSMIC Settings → Panel, and that
it survives a restart, requires manual verification against a running
COSMIC session — not an automated test in this repo.

- **AC-1** [REQ-001/TS-002/TS-003/TS-004]: Given Caps Lock is off and the
  applet is running, when the sysfs capslock LED brightness becomes
  non-zero, then the panel shows "CapsLockOn" within one polling interval
  with no further user action.
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac1_state_reads_on_when_led_brightness_becomes_nonzero` — red
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac1_state_is_on_if_any_of_multiple_led_nodes_is_nonzero` — red
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac1_finds_only_input_capslock_led_brightness_files` — red
- **AC-2** [REQ-002/TS-002/TS-003/TS-004]: Given Caps Lock is on and
  "CapsLockOn" is visible, when the sysfs capslock LED brightness returns
  to zero, then the text is removed from the panel within one polling
  interval, with no placeholder or dimmed element left in its place.
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac2_state_reads_off_when_led_brightness_returns_to_zero` — red
- **AC-3** [REQ-003/TS-003]: Given the applet is running and the panel is
  not restarted, when Caps Lock is toggled on and off repeatedly, then
  the panel display tracks every toggle without the user restarting the
  panel, logging out, or otherwise intervening.
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac3_tracks_every_toggle_independently_with_no_stale_caching` — red
- **AC-4** [REQ-004/TS-007]: Given the applet's settings popup is open,
  when the user selects a different font family from the dropdown, then
  the visible "CapsLockOn" text immediately re-renders in that font.
  - Test: `capsIndicator/src/settings.rs::tests::ac4_render_label_style_uses_updated_font_family` — red
- **AC-5** [REQ-005/TS-007]: Given the applet's settings popup is open,
  when the user enters a new hex color value, then the visible
  "CapsLockOn" text immediately re-renders in that color.
  - Test: `capsIndicator/src/settings.rs::tests::ac5_render_label_style_uses_updated_text_color` — red
- **AC-6** [REQ-006/TS-007]: Given the applet's settings popup is open,
  when the user changes the text-size control, then the visible
  "CapsLockOn" text immediately re-renders at that size.
  - Test: `capsIndicator/src/settings.rs::tests::ac6_render_label_style_uses_updated_text_size` — red
- **AC-7** [REQ-007/TS-001/TS-006]: Given the applet is registered as a
  standard cosmic-panel plugin, when the user opens COSMIC Settings →
  Panel and drags the applet's entry into the Start, Center, or End
  column, then `cosmic-panel` positions the applet in that chosen zone.
  - Test: `capsIndicator/src/desktop_entry.rs::tests::ac7_well_formed_desktop_entry_is_registerable_as_a_panel_applet` — red
  - Test: `capsIndicator/src/desktop_entry.rs::tests::ac7_desktop_entry_missing_exec_is_not_registerable` — red
  - Test: `capsIndicator/src/desktop_entry.rs::tests::ac7_desktop_entry_with_mismatched_app_id_is_not_registerable` — red
  - Manual verification (not an automated test, per the note above): drag
    the installed applet between the Start/Center/End columns in COSMIC
    Settings → Panel on a running COSMIC session and confirm it moves.
- **AC-8** [REQ-008/TS-005]: Given the user has changed font, color, or
  text size in the settings popup, when the applet or panel restarts (or
  the user logs back in), then "CapsLockOn" renders using the same font,
  color, and size as before the restart.
  - Test: `capsIndicator/src/settings.rs::tests::ac8_settings_round_trip_preserves_font_color_size` — red
  - Test: `capsIndicator/src/settings.rs::tests::ac8_render_label_style_reflects_settings_restored_after_a_round_trip` — red
- **AC-9** [REQ-007/REQ-008/TS-006]: Given the user has moved the applet
  to a given panel zone via COSMIC Settings → Panel, when the panel
  restarts or the user logs back in, then the applet still launches in
  that same zone (persistence here is owned by `cosmic-panel`'s own
  configuration, per TS-006).
  - Test: `capsIndicator/src/desktop_entry.rs::tests::ac9_desktop_entry_declares_the_stable_expected_app_id` — red
  - Manual verification (not an automated test, per the note above):
    confirm the applet stays in its chosen zone after a panel
    restart/relogin on a running COSMIC session.
- **AC-10** [REQ-003/TS-008]: Given the running system has no
  `/sys/class/leds/input*::capslock` node, when the applet starts, then
  it logs exactly one warning to stderr/the journal and renders no
  visible content or error state in the panel.
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac10_state_is_unavailable_when_no_led_paths_found` — red
  - Test: `capsIndicator/src/caps_detect.rs::tests::ac10_startup_warning_present_only_when_unavailable` — red
