//! Persisted applet settings (TS-005): a `cosmic_config::CosmicConfigEntry`
//! that mirrors the already-tested, dependency-free
//! [`crate::settings::IndicatorSettings`] one-for-one. This module owns
//! disk persistence only — rendering logic stays in `settings::render_label_style`.

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

use crate::settings::IndicatorSettings;

#[derive(Debug, Clone, PartialEq, CosmicConfigEntry, serde::Deserialize, serde::Serialize)]
#[version = 1]
pub struct AppletConfig {
    pub font_family: String,
    pub text_color: String,
    pub text_size: f32,
}

impl Default for AppletConfig {
    fn default() -> Self {
        Self {
            font_family: "Noto Sans".to_string(),
            text_color: "#FFFFFF".to_string(),
            text_size: 14.0,
        }
    }
}

impl From<&AppletConfig> for IndicatorSettings {
    fn from(config: &AppletConfig) -> Self {
        Self {
            font_family: config.font_family.clone(),
            text_color: config.text_color.clone(),
            text_size: config.text_size,
        }
    }
}
