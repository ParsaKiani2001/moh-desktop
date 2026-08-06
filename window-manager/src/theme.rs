use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]  // ✅ Deserialize اضافه شد
pub struct Theme {
    #[serde(default = "default_height")]
    pub titlebar_height: u16,
    #[serde(default = "default_bg")]
    pub titlebar_bg: String,
    #[serde(default = "default_inactive")]
    pub titlebar_inactive_bg: String,
    #[serde(default = "default_text")]
    pub titlebar_text: String,
    #[serde(default = "default_close")]
    pub btn_close_color: String,
    #[serde(default = "default_min")]
    pub btn_min_color: String,
    #[serde(default = "default_max")]
    pub btn_max_color: String,
    #[serde(default = "default_btn_size")]
    pub btn_size: u16,
    #[serde(default = "default_padding")]
    pub btn_padding: u16,
}

fn default_height() -> u16 { 30 }
fn default_bg() -> String { "#2d5da1".into() }
fn default_inactive() -> String { "#555555".into() }
fn default_text() -> String { "#ffffff".into() }
fn default_close() -> String { "#ff5f57".into() }
fn default_min() -> String { "#ffbd2e".into() }
fn default_max() -> String { "#28c840".into() }
fn default_btn_size() -> u16 { 16 }
fn default_padding() -> u16 { 8 }

impl Default for Theme {
    fn default() -> Self {
        Self {
            titlebar_height: default_height(),
            titlebar_bg: default_bg(),
            titlebar_inactive_bg: default_inactive(),
            titlebar_text: default_text(),
            btn_close_color: default_close(),
            btn_min_color: default_min(),
            btn_max_color: default_max(),
            btn_size: default_btn_size(),
            btn_padding: default_padding(),
        }
    }
}

impl Theme {
    pub fn load() -> Self {
        let text = match fs::read_to_string("desktop.toml") {
            Ok(t) => t,
            Err(_) => return Self::default(),
        };

        let config: toml::Value = match toml::from_str(&text) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };

        // ✅ استفاده از get و try_into
        if let Some(theme_value) = config.get("theme") {
            theme_value.clone().try_into().unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn parse_color(&self, hex: &str) -> u32 {
        let hex = hex.trim_start_matches('#');
        u32::from_str_radix(hex, 16).unwrap_or(0x000000)
    }

    pub fn titlebar_bg(&self) -> u32 {
        self.parse_color(&self.titlebar_bg)
    }

    pub fn titlebar_inactive_bg(&self) -> u32 {
        self.parse_color(&self.titlebar_inactive_bg)
    }

    pub fn titlebar_text(&self) -> u32 {
        self.parse_color(&self.titlebar_text)
    }

    pub fn btn_close(&self) -> u32 {
        self.parse_color(&self.btn_close_color)
    }

    pub fn btn_min(&self) -> u32 {
        self.parse_color(&self.btn_min_color)
    }

    pub fn btn_max(&self) -> u32 {
        self.parse_color(&self.btn_max_color)
    }
}