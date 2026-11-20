//! The real `cosmic::Application`/Wayland GUI layer (TS-001/TS-004/TS-007).
//! This module is the only place that talks to `libcosmic`/`cosmic-config`;
//! all state rendering and (de)serialization logic is delegated to the
//! pure, already-tested `caps_detect` and `settings` modules.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::{
    Alignment, Background, Color, Length, Subscription,
    futures::channel::mpsc,
    stream,
    window,
};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::destroy_popup;
use cosmic::widget::{button, column, container, dropdown, row, slider, text, text_input};
use cosmic::{Application, Element};

use crate::caps_detect::{self, CapsLockState};
use crate::config::AppletConfig;
use crate::settings;

const LABEL_TEXT: &str = "CapsLockOn";
const LEDS_DIR: &str = "/sys/class/leds";

pub struct CapsLockApplet {
    core: Core,
    popup: Option<window::Id>,
    caps_state: CapsLockState,
    config_handler: Option<Config>,
    config: AppletConfig,
    color_input: String,
    font_families: Vec<String>,
    font_name_cache: HashMap<String, &'static str>,
    current_font: cosmic::iced::Font,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(window::Id),
    CapsLockState(CapsLockState),
    ConfigChanged(AppletConfig),
    FontSelected(usize),
    ColorInput(String),
    SizeChanged(f32),
}

impl CapsLockApplet {
    fn indicator_settings(&self) -> settings::IndicatorSettings {
        settings::IndicatorSettings::from(&self.config)
    }

    fn sync_font(&mut self) {
        let family = self.config.font_family.clone();
        let name = static_font_name(&mut self.font_name_cache, &family);
        self.current_font = cosmic::iced::Font {
            family: cosmic::iced::font::Family::Name(name),
            ..cosmic::iced::Font::DEFAULT
        };
    }

    fn set_font_family(&mut self, value: String) {
        match &self.config_handler {
            Some(handler) => {
                if let Err(err) = self.config.set_font_family(handler, value) {
                    tracing::error!(?err, "failed to persist capslock applet font_family");
                }
            }
            None => self.config.font_family = value,
        }
        self.sync_font();
    }

    fn set_text_color(&mut self, value: String) {
        match &self.config_handler {
            Some(handler) => {
                if let Err(err) = self.config.set_text_color(handler, value) {
                    tracing::error!(?err, "failed to persist capslock applet text_color");
                }
            }
            None => self.config.text_color = value,
        }
    }

    fn set_text_size(&mut self, value: f32) {
        match &self.config_handler {
            Some(handler) => {
                if let Err(err) = self.config.set_text_size(handler, value) {
                    tracing::error!(?err, "failed to persist capslock applet text_size");
                }
            }
            None => self.config.text_size = value,
        }
    }
}

impl Application for CapsLockApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = crate::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let config_handler = Config::new(crate::APP_ID, AppletConfig::VERSION).ok();
        let config = config_handler
            .as_ref()
            .map(|handler| {
                AppletConfig::get_entry(handler).unwrap_or_else(|(errors, config)| {
                    for err in errors {
                        tracing::error!(?err, "error loading capslock applet config");
                    }
                    config
                })
            })
            .unwrap_or_default();

        let initial_paths = caps_detect::find_capslock_led_paths(Path::new(LEDS_DIR));
        let startup_state = if initial_paths.is_empty() {
            CapsLockState::Unavailable
        } else {
            caps_detect::read_capslock_state(&initial_paths)
        };
        if let Some(warning) = caps_detect::startup_warning(
            if initial_paths.is_empty() {
                CapsLockState::Unavailable
            } else {
                CapsLockState::Off
            },
        ) {
            tracing::warn!("{warning}");
        }

        let mut app = Self {
            core,
            popup: None,
            caps_state: startup_state,
            color_input: config.text_color.clone(),
            font_families: enumerate_font_families(),
            font_name_cache: HashMap::new(),
            current_font: cosmic::iced::Font::DEFAULT,
            config_handler,
            config,
        };
        app.sync_font();

        (app, Task::none())
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.core.watch_config(crate::APP_ID).map(|update| {
                for err in update.errors {
                    tracing::error!(?err, "error watching capslock applet config");
                }
                Message::ConfigChanged(update.config)
            }),
            caps_lock_subscription(),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePopup => {
                if let Some(id) = self.popup.take() {
                    return destroy_popup(id);
                }
                return cosmic::surface::surface_task(cosmic::surface::action::app_popup(
                    |_| Default::default(),
                    |app: &mut Self| {
                        let new_id = window::Id::unique();
                        app.popup = Some(new_id);
                        app.core.applet.get_popup_settings(
                            app.core.main_window_id().unwrap(),
                            new_id,
                            None,
                            None,
                            None,
                        )
                    },
                    None,
                ));
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::CapsLockState(state) => {
                self.caps_state = state;
            }
            Message::ConfigChanged(config) => {
                self.color_input = config.text_color.clone();
                self.config = config;
                self.sync_font();
            }
            Message::FontSelected(index) => {
                if let Some(family) = self.font_families.get(index).cloned() {
                    self.set_font_family(family);
                }
            }
            Message::ColorInput(value) => {
                self.color_input = value.clone();
                if parse_hex_color(&value).is_some() {
                    self.set_text_color(value);
                }
            }
            Message::SizeChanged(size) => {
                self.set_text_size(size);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = if self.caps_state == CapsLockState::On {
            let style = settings::render_label_style(&self.indicator_settings());
            let color = parse_hex_color(&style.color).unwrap_or(Color::WHITE);
            text(LABEL_TEXT)
                .size(style.size)
                .class(color)
                .font(self.current_font)
                .into()
        } else {
            cosmic::iced::widget::Space::new()
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0))
                .into()
        };

        button::custom(content)
            .on_press_down(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon)
            .into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        let selected_index = self
            .font_families
            .iter()
            .position(|f| f == &self.config.font_family);

        let swatch_color = parse_hex_color(&self.color_input).unwrap_or(Color::BLACK);
        let swatch = container(
            cosmic::iced::widget::Space::new()
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0)),
        )
        .style(move |_theme| cosmic::iced::widget::container::Style {
            background: Some(Background::Color(swatch_color)),
            ..Default::default()
        });

        let content = column![
            text::heading("CapsLock Indicator"),
            row![
                text("Font"),
                dropdown(
                    Cow::Borrowed(self.font_families.as_slice()),
                    selected_index,
                    Message::FontSelected,
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Color"),
                text_input("#RRGGBB", self.color_input.as_str()).on_input(Message::ColorInput),
                swatch,
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Size"),
                slider(8.0..=32.0, self.config.text_size, Message::SizeChanged),
                text(format!("{:.0}px", self.config.text_size)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(12)
        .padding(12);

        self.core.applet.popup_container(content).into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }
}

/// Bridges a dedicated background polling thread (TS-003) into the
/// `iced` subscription system: the thread owns the 100ms poll loop and
/// calls into the already-tested `caps_detect` functions, sending a
/// `Message` only when the combined state actually changes.
fn caps_lock_subscription() -> Subscription<Message> {
    Subscription::run_with("capslock-poll", |_id| {
        stream::channel(8, |output: mpsc::Sender<Message>| async move {
            std::thread::spawn(move || {
                let mut output = output;
                let leds_dir = Path::new(LEDS_DIR);
                let mut last_state: Option<CapsLockState> = None;
                loop {
                    let paths = caps_detect::find_capslock_led_paths(leds_dir);
                    let state = caps_detect::read_capslock_state(&paths);
                    if last_state != Some(state) {
                        last_state = Some(state);
                        if output.try_send(Message::CapsLockState(state)).is_err() {
                            break;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });

            std::future::pending::<()>().await
        })
    })
}

/// Enumerates installed font family names (TS-007/CON-002) via `fontdb`'s
/// system font loader — the same underlying mechanism `cosmic_text`
/// re-exports, without pulling in that crate's much heavier default
/// dependency set (shaping/rasterization libs this applet never uses).
fn enumerate_font_families() -> Vec<String> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut families: Vec<String> = db
        .faces()
        .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
        .collect();
    families.sort();
    families.dedup();
    families
}

/// `iced::Font::family` needs a `&'static str`, but font families are
/// user-chosen at runtime, so each distinct family name seen is leaked
/// once and cached — a bounded, one-time-per-family allocation rather
/// than a leak per render.
fn static_font_name(cache: &mut HashMap<String, &'static str>, name: &str) -> &'static str {
    if let Some(&cached) = cache.get(name) {
        return cached;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    cache.insert(name.to_string(), leaked);
    leaked
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::from_rgb8(r, g, b))
}
