---
title: CapsLock Indicator Panel Extension
date_created: 2026-11-20
status: Draft
---

# CapsLock Indicator Panel Extension

## Purpose & Scope
A COSMIC desktop environment panel applet (extension) that shows the text
"CapsLockOn" in the user's COSMIC panel whenever the Caps Lock key is
active, and removes it from the panel whenever Caps Lock is inactive. The
extension also exposes user-configurable display options: the font used
for the indicator text, the text's color, the text's size, and the
indicator's placement on the panel.

Out of scope: indicators for other lock keys (Num Lock, Scroll Lock),
system tray/notification-area icons, toast/popup notifications on state
change, and support for desktop environments other than COSMIC.

## Functional Requirements
- **REQ-001**: The extension MUST display the text "CapsLockOn" in the
  COSMIC panel whenever Caps Lock is active.
- **REQ-002**: The extension MUST remove the "CapsLockOn" text from the
  panel whenever Caps Lock is inactive (no placeholder or dimmed state is
  shown while inactive).
- **REQ-003**: The extension MUST detect Caps Lock state changes while
  running and update the panel display accordingly, without requiring the
  user to restart the panel or log out.
- **REQ-004**: The user MUST be able to set the font used for the
  indicator text, chosen from the fonts currently installed on the system.
- **REQ-005**: The user MUST be able to set the color of the indicator
  text.
- **REQ-006**: The user MUST be able to set the size of the indicator
  text.
- **REQ-007**: The user MUST be able to choose which panel zone the
  indicator sits in: left, center, or right.
- **REQ-008**: The user's font, color, size, and placement settings MUST
  persist across panel restarts and logout/login (i.e., are not reset to
  defaults each session).

## Constraints
- **CON-001**: Must run as a COSMIC desktop environment panel applet
  (target platform: COSMIC DE's `cosmic-panel`), not as a standalone
  application or tray icon.
- **CON-002**: Font selection is limited to fonts already installed on
  the system — the extension does not bundle or download fonts.

## Key Entities
- **IndicatorSettings** — the extension's user-configurable state:
  - `fontFamily`: which installed font renders the indicator text.
  - `textColor`: the color of the indicator text.
  - `textSize`: the size of the indicator text.
  - `placement`: which panel zone the indicator sits in — left, center,
    or right.

## Success Criteria
- **SC-001**: Pressing Caps Lock on makes "CapsLockOn" appear in the panel
  without any user action beyond pressing the key.
- **SC-002**: Pressing Caps Lock off removes the text from the panel
  without any user action beyond pressing the key.
- **SC-003**: Changing the font, color, size, or placement setting visibly
  changes the indicator's appearance/position without requiring a logout
  or panel restart.
- **SC-004**: After a panel restart or a fresh login, the indicator still
  reflects the previously chosen font, color, size, and placement.

## Assumptions
- "Top bar" refers to the user's COSMIC panel (`cosmic-panel`), on
  whichever screen edge it is docked — not necessarily the literal top of
  the screen.
- Caps Lock state is read as a single system-wide state (as reflected by
  the keyboard LED), not tracked separately per physical keyboard device,
  since the user's request describes one indicator reacting to "the CAPS
  key."
- The exact default values for font/color/size/placement before the user
  changes anything are left to the technical spec; this spec only
  requires that the settings be changeable and persistent.
- How the user reaches these settings (e.g., a popup/context menu on the
  applet vs. a `cosmic-settings` page) is a technical design decision,
  deferred to the technical spec — this spec only requires that the
  settings exist and are user-reachable.
