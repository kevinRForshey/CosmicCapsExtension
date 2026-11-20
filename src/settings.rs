//! `IndicatorSettings` (the applet's configurable font/color/size state,
//! TS-005) and the pure logic that renders it (TS-007) and round-trips
//! it through text (the persistence contract a real `cosmic-config`
//! on-disk store must preserve for REQ-008).

#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorSettings {
    pub font_family: String,
    pub text_color: String,
    pub text_size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStyle {
    pub font_family: String,
    pub color: String,
    pub size: f32,
}

/// Computes the panel label's style directly from the current settings,
/// so a settings change is reflected the next time this is called — no
/// cached/stale style survives an update (TS-007).
pub fn render_label_style(settings: &IndicatorSettings) -> LabelStyle {
    LabelStyle {
        font_family: settings.font_family.clone(),
        color: settings.text_color.clone(),
        size: settings.text_size,
    }
}

pub fn serialize_settings(settings: &IndicatorSettings) -> String {
    format!(
        "font_family={}\ntext_color={}\ntext_size={}\n",
        settings.font_family, settings.text_color, settings.text_size
    )
}

/// Parses the `key=value` text `serialize_settings` produces. Returns
/// `None` if any of the three fields is missing or malformed.
pub fn deserialize_settings(data: &str) -> Option<IndicatorSettings> {
    let mut font_family = None;
    let mut text_color = None;
    let mut text_size = None;

    for line in data.lines() {
        let (key, value) = line.split_once('=')?;
        match key {
            "font_family" => font_family = Some(value.to_string()),
            "text_color" => text_color = Some(value.to_string()),
            "text_size" => text_size = value.parse::<f32>().ok(),
            _ => {}
        }
    }

    Some(IndicatorSettings {
        font_family: font_family?,
        text_color: text_color?,
        text_size: text_size?,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn ac4_render_label_style_uses_updated_font_family() {
        let mut settings = super::IndicatorSettings {
            font_family: "Inter".into(),
            text_color: "#000000".into(),
            text_size: 12.0,
        };
        assert_eq!(super::render_label_style(&settings).font_family, "Inter");

        settings.font_family = "JetBrains Mono".into();
        assert_eq!(
            super::render_label_style(&settings).font_family,
            "JetBrains Mono"
        );
    }

    #[test]
    fn ac5_render_label_style_uses_updated_text_color() {
        let mut settings = super::IndicatorSettings {
            font_family: "Inter".into(),
            text_color: "#000000".into(),
            text_size: 12.0,
        };
        assert_eq!(super::render_label_style(&settings).color, "#000000");

        settings.text_color = "#ABCDEF".into();
        assert_eq!(super::render_label_style(&settings).color, "#ABCDEF");
    }

    #[test]
    fn ac6_render_label_style_uses_updated_text_size() {
        let mut settings = super::IndicatorSettings {
            font_family: "Inter".into(),
            text_color: "#000000".into(),
            text_size: 12.0,
        };
        assert_eq!(super::render_label_style(&settings).size, 12.0);

        settings.text_size = 24.0;
        assert_eq!(super::render_label_style(&settings).size, 24.0);
    }

    #[test]
    fn ac8_settings_round_trip_preserves_font_color_size() {
        let original = super::IndicatorSettings {
            font_family: "Fira Code".to_string(),
            text_color: "#FF8800".to_string(),
            text_size: 18.0,
        };

        let serialized = super::serialize_settings(&original);
        let restored = super::deserialize_settings(&serialized).expect("should deserialize");

        assert_eq!(restored, original);
    }

    #[test]
    fn ac8_render_label_style_reflects_settings_restored_after_a_round_trip() {
        let original = super::IndicatorSettings {
            font_family: "Noto Sans".into(),
            text_color: "#112233".into(),
            text_size: 14.0,
        };

        let restored =
            super::deserialize_settings(&super::serialize_settings(&original)).unwrap();
        let style = super::render_label_style(&restored);

        assert_eq!(style.font_family, "Noto Sans");
        assert_eq!(style.color, "#112233");
        assert_eq!(style.size, 14.0);
    }
}
