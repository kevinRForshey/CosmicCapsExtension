# CapsLock Indicator Panel Extension — Stories

## US-001: Detect Caps Lock state and show/hide the panel indicator
- **Covers**: AC-1, AC-2, AC-3, AC-10 (REQ-001/REQ-002/REQ-003, TS-002/TS-003/TS-004/TS-008)
- **Tests**:
  - `capsIndicator/src/caps_detect.rs::tests::ac1_state_reads_on_when_led_brightness_becomes_nonzero`
  - `capsIndicator/src/caps_detect.rs::tests::ac1_state_is_on_if_any_of_multiple_led_nodes_is_nonzero`
  - `capsIndicator/src/caps_detect.rs::tests::ac1_finds_only_input_capslock_led_brightness_files`
  - `capsIndicator/src/caps_detect.rs::tests::ac2_state_reads_off_when_led_brightness_returns_to_zero`
  - `capsIndicator/src/caps_detect.rs::tests::ac3_tracks_every_toggle_independently_with_no_stale_caching`
  - `capsIndicator/src/caps_detect.rs::tests::ac10_state_is_unavailable_when_no_led_paths_found`
  - `capsIndicator/src/caps_detect.rs::tests::ac10_startup_warning_present_only_when_unavailable`
- **Status**: Passing
- **Priority**: P1
- **Story**: As a COSMIC desktop user, I want the panel to show
  "CapsLockOn" exactly while Caps Lock is active and hide it the instant
  Caps Lock turns off, so that I always have an at-a-glance, always-current
  indication of my Caps Lock state without needing to press a key to check.
- **Acceptance criteria**:
  - **AC-1**: Given Caps Lock is off and the applet is running, when the
    sysfs capslock LED brightness becomes non-zero, then the panel shows
    "CapsLockOn" within one polling interval (~100ms) with no further
    user action.
  - **AC-2**: Given Caps Lock is on and "CapsLockOn" is visible, when the
    sysfs capslock LED brightness returns to zero, then the text is
    removed from the panel within one polling interval, with no
    placeholder or dimmed element left in its place.
  - **AC-3**: Given the applet is running and the panel is not restarted,
    when Caps Lock is toggled on and off repeatedly, then the panel
    display tracks every toggle without the user restarting the panel,
    logging out, or otherwise intervening.
  - **AC-10**: Given the running system has no
    `/sys/class/leds/input*::capslock` node, when the applet starts, then
    it logs exactly one warning to stderr/the journal and renders no
    visible content or error state in the panel.

## US-002: Register the applet as a standard, placeable cosmic-panel plugin
- **Covers**: AC-7, AC-9 (REQ-007/REQ-008, TS-001/TS-006)
- **Tests**:
  - `capsIndicator/src/desktop_entry.rs::tests::ac7_well_formed_desktop_entry_is_registerable_as_a_panel_applet`
  - `capsIndicator/src/desktop_entry.rs::tests::ac7_desktop_entry_missing_exec_is_not_registerable`
  - `capsIndicator/src/desktop_entry.rs::tests::ac7_desktop_entry_with_mismatched_app_id_is_not_registerable`
  - `capsIndicator/src/desktop_entry.rs::tests::ac9_desktop_entry_declares_the_stable_expected_app_id`
- **Status**: Passing
- **Priority**: P1
- **Story**: As a COSMIC desktop user, I want the CapsLock indicator
  applet to register as a standard, stably-identified panel plugin, so
  that I can place and keep it in whichever panel zone (left, center, or
  right) I prefer, the same way I arrange every other panel applet.
- **Acceptance criteria**:
  - **AC-7**: Given the applet is registered as a standard cosmic-panel
    plugin, when the user opens COSMIC Settings → Panel and drags the
    applet's entry into the Start, Center, or End column, then
    `cosmic-panel` positions the applet in that chosen zone.
  - **AC-9**: Given the user has moved the applet to a given panel zone
    via COSMIC Settings → Panel, when the panel restarts or the user logs
    back in, then the applet still launches in that same zone
    (persistence here is owned by `cosmic-panel`'s own configuration, per
    TS-006).

## US-003: Configure and persist the indicator's font, color, and size
- **Covers**: AC-4, AC-5, AC-6, AC-8 (REQ-004/REQ-005/REQ-006/REQ-008, TS-005/TS-007)
- **Tests**:
  - `capsIndicator/src/settings.rs::tests::ac4_render_label_style_uses_updated_font_family`
  - `capsIndicator/src/settings.rs::tests::ac5_render_label_style_uses_updated_text_color`
  - `capsIndicator/src/settings.rs::tests::ac6_render_label_style_uses_updated_text_size`
  - `capsIndicator/src/settings.rs::tests::ac8_settings_round_trip_preserves_font_color_size`
  - `capsIndicator/src/settings.rs::tests::ac8_render_label_style_reflects_settings_restored_after_a_round_trip`
- **Status**: Passing
- **Priority**: P2
- **Story**: As a COSMIC desktop user, I want to choose the indicator's
  font, color, and text size, and have those choices persist across
  restarts, so that the indicator fits my panel's look and keeps that
  look without me reconfiguring it every session.
- **Acceptance criteria**:
  - **AC-4**: Given the applet's settings popup is open, when the user
    selects a different font family from the dropdown, then the visible
    "CapsLockOn" text immediately re-renders in that font.
  - **AC-5**: Given the applet's settings popup is open, when the user
    enters a new hex color value, then the visible "CapsLockOn" text
    immediately re-renders in that color.
  - **AC-6**: Given the applet's settings popup is open, when the user
    changes the text-size control, then the visible "CapsLockOn" text
    immediately re-renders at that size.
  - **AC-8**: Given the user has changed font, color, or text size in the
    settings popup, when the applet or panel restarts (or the user logs
    back in), then "CapsLockOn" renders using the same font, color, and
    size as before the restart.
