//! Caps Lock state detection via the Linux sysfs keyboard-LED class
//! (TS-002/TS-003/TS-008). A system-wide boolean is derived by OR-ing
//! every matched `input*::capslock/brightness` node, so the applet
//! doesn't need per-keyboard tracking (see spec.md's assumption).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsLockState {
    On,
    Off,
    Unavailable,
}

/// Finds sysfs capslock LED brightness files under `base_dir` (in
/// production, `/sys/class/leds`) matching `input*::capslock/brightness`.
pub fn find_capslock_led_paths(base_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(base_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter(|entry| is_capslock_led_dir_name(&entry.file_name()))
        .map(|entry| entry.path().join("brightness"))
        .collect()
}

fn is_capslock_led_dir_name(name: &OsStr) -> bool {
    let name = name.to_string_lossy();
    name.starts_with("input") && name.ends_with("::capslock")
}

/// Reads the given LED brightness files and combines them into one
/// system-wide state: `Unavailable` if `paths` is empty (TS-008), `On` if
/// any file holds a non-zero value, `Off` otherwise.
pub fn read_capslock_state(paths: &[PathBuf]) -> CapsLockState {
    if paths.is_empty() {
        return CapsLockState::Unavailable;
    }

    let any_on = paths.iter().any(|path| {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| content.trim().parse::<u32>().ok())
            .is_some_and(|value| value != 0)
    });

    if any_on {
        CapsLockState::On
    } else {
        CapsLockState::Off
    }
}

/// A single startup warning to log when no capslock LED node exists at
/// all (TS-008); `None` for a normal `On`/`Off` reading.
pub fn startup_warning(state: CapsLockState) -> Option<String> {
    match state {
        CapsLockState::Unavailable => Some(
            "no /sys/class/leds/input*::capslock node found; Caps Lock \
             state cannot be detected on this system"
                .to_string(),
        ),
        CapsLockState::On | CapsLockState::Off => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn make_temp_dir(label: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("caps_detect_test_{label}_{n}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ac1_state_reads_on_when_led_brightness_becomes_nonzero() {
        let dir = make_temp_dir("ac1");
        let brightness = dir.join("brightness");
        fs::write(&brightness, "0\n").unwrap();
        let paths = vec![brightness.clone()];

        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::Off
        );

        fs::write(&brightness, "1\n").unwrap();
        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::On
        );

        cleanup(&dir);
    }

    #[test]
    fn ac1_state_is_on_if_any_of_multiple_led_nodes_is_nonzero() {
        let dir = make_temp_dir("ac1_multi");
        let b1 = dir.join("kbd1_brightness");
        let b2 = dir.join("kbd2_brightness");
        fs::write(&b1, "0").unwrap();
        fs::write(&b2, "0").unwrap();
        let paths = vec![b1.clone(), b2.clone()];

        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::Off
        );

        fs::write(&b2, "1").unwrap();
        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::On
        );

        cleanup(&dir);
    }

    #[test]
    fn ac1_finds_only_input_capslock_led_brightness_files() {
        let dir = make_temp_dir("find_paths");
        let good1 = dir.join("input0::capslock");
        let good2 = dir.join("input3::capslock");
        let bad_numlock = dir.join("input0::numlock");
        let bad_other = dir.join("phy0::capslock");
        for d in [&good1, &good2, &bad_numlock, &bad_other] {
            fs::create_dir_all(d).unwrap();
            fs::write(d.join("brightness"), "0").unwrap();
        }

        let mut found = super::find_capslock_led_paths(&dir);
        found.sort();
        let mut expected = vec![good1.join("brightness"), good2.join("brightness")];
        expected.sort();
        assert_eq!(found, expected);

        cleanup(&dir);
    }

    #[test]
    fn ac2_state_reads_off_when_led_brightness_returns_to_zero() {
        let dir = make_temp_dir("ac2");
        let brightness = dir.join("brightness");
        fs::write(&brightness, "1\n").unwrap();
        let paths = vec![brightness.clone()];

        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::On
        );

        fs::write(&brightness, "0\n").unwrap();
        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::Off
        );

        cleanup(&dir);
    }

    #[test]
    fn ac3_tracks_every_toggle_independently_with_no_stale_caching() {
        let dir = make_temp_dir("ac3");
        let brightness = dir.join("brightness");
        let paths = vec![brightness.clone()];
        let sequence = [0, 1, 0, 1, 1, 0];

        for &value in &sequence {
            fs::write(&brightness, value.to_string()).unwrap();
            let expected = if value != 0 {
                super::CapsLockState::On
            } else {
                super::CapsLockState::Off
            };
            assert_eq!(
                super::read_capslock_state(&paths),
                expected,
                "mismatch at value {value}"
            );
        }

        cleanup(&dir);
    }

    #[test]
    fn ac10_state_is_unavailable_when_no_led_paths_found() {
        let paths: Vec<PathBuf> = vec![];
        assert_eq!(
            super::read_capslock_state(&paths),
            super::CapsLockState::Unavailable
        );
    }

    #[test]
    fn ac10_startup_warning_present_only_when_unavailable() {
        assert!(super::startup_warning(super::CapsLockState::Unavailable).is_some());
        assert!(super::startup_warning(super::CapsLockState::On).is_none());
        assert!(super::startup_warning(super::CapsLockState::Off).is_none());
    }
}
