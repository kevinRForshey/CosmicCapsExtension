//! Validates this applet's `.desktop` entry (TS-001/TS-006) — the
//! structural precondition for `cosmic-panel` to register it as a
//! draggable entry in COSMIC Settings -> Panel.
//!
//! Note: the actual end-to-end behavior of AC-7/AC-9 (dragging the
//! applet between panel zones in a running COSMIC session, and that
//! choice surviving a restart) is owned by `cosmic-panel` itself, not
//! this crate (see tech-spec.md TS-006) and is verified manually, not by
//! these tests. These tests only cover the structural precondition this
//! crate is responsible for: shipping a well-formed, stably-identified
//! applet `.desktop` entry for `cosmic-panel` to register.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopEntryCheck {
    pub has_type_application: bool,
    pub has_exec: bool,
    pub app_id_is_present_and_matches: bool,
}

/// Parses a `.desktop` entry's raw text and checks it against the
/// structural requirements `cosmic-panel` needs to register this applet
/// under `expected_app_id`.
pub fn validate_desktop_entry(contents: &str, expected_app_id: &str) -> DesktopEntryCheck {
    let has_type_application = contents
        .lines()
        .any(|line| line.trim() == "Type=Application");

    let exec_value = contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("Exec="));

    let has_exec = exec_value.is_some_and(|value| !value.trim().is_empty());

    let app_id_is_present_and_matches = exec_value
        .and_then(|value| value.trim().rsplit('/').next())
        .is_some_and(|basename| basename == expected_app_id);

    DesktopEntryCheck {
        has_type_application,
        has_exec,
        app_id_is_present_and_matches,
    }
}

/// `true` only when the entry declares `Type=Application`, a non-empty
/// `Exec=`, and an `Exec=` basename matching the expected, stable app-id.
pub fn is_registerable_cosmic_panel_applet(check: &DesktopEntryCheck) -> bool {
    check.has_type_application && check.has_exec && check.app_id_is_present_and_matches
}

#[cfg(test)]
mod tests {
    const APP_ID: &str = "com.example.CosmicAppletCapsLock";

    #[test]
    fn ac7_well_formed_desktop_entry_is_registerable_as_a_panel_applet() {
        let contents = format!(
            "[Desktop Entry]\nType=Application\nName=CapsLock Indicator\nExec=/usr/bin/{APP_ID}\nNoDisplay=true\n"
        );

        let check = super::validate_desktop_entry(&contents, APP_ID);
        assert!(super::is_registerable_cosmic_panel_applet(&check));
    }

    #[test]
    fn ac7_desktop_entry_missing_exec_is_not_registerable() {
        let contents = "[Desktop Entry]\nType=Application\nName=CapsLock Indicator\n";

        let check = super::validate_desktop_entry(contents, APP_ID);
        assert!(!super::is_registerable_cosmic_panel_applet(&check));
    }

    #[test]
    fn ac7_desktop_entry_with_mismatched_app_id_is_not_registerable() {
        let contents = "[Desktop Entry]\nType=Application\nExec=/usr/bin/com.example.SomeOtherApplet\n";

        let check = super::validate_desktop_entry(contents, APP_ID);
        assert!(!super::is_registerable_cosmic_panel_applet(&check));
    }

    #[test]
    fn ac9_desktop_entry_declares_the_stable_expected_app_id() {
        let contents = format!("[Desktop Entry]\nType=Application\nExec=/usr/bin/{APP_ID}\n");

        let check = super::validate_desktop_entry(&contents, APP_ID);
        assert!(check.app_id_is_present_and_matches);
    }
}
