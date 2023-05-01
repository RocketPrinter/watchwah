use std::fs;
use eframe::CreationContext;
use eframe::egui::Visuals;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use common::get_config_path;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClientConfig {
    // todo: key

    #[serde(default)]
    pub theme: Theme,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Auto,
    DefaultLight,
    DefaultDark,
    Frappe,
    Latte,
    Macchiato,
    Mocha
}

impl Theme {
    pub fn set(self, cc: &CreationContext) {
        use crate::client_config::Theme::*;
        use catppuccin_egui::*;
        match self {
            Auto => match cc.integration_info.system_theme {
                None | Some(eframe::Theme::Dark) => Self::set(Mocha, cc),
                Some(eframe::Theme::Light) => Self::set(Latte, cc),
            }
            DefaultLight => cc.egui_ctx.set_visuals(Visuals::light()),
            DefaultDark => cc.egui_ctx.set_visuals(Visuals::dark()),
            Frappe => set_theme(&cc.egui_ctx, FRAPPE),
            Latte => set_theme(&cc.egui_ctx, LATTE),
            Macchiato => set_theme(&cc.egui_ctx, MACCHIATO),
            Mocha => set_theme(&cc.egui_ctx, MOCHA),
        }
    }
}

pub fn load_config() -> ClientConfig {
    match fs::read_to_string(get_config_path().join("client.toml")) {
        Ok(file) => {
            toml::from_str(&file).map_err(|e| error!("Failed to parse config: {e}")).unwrap()
        }
        Err(err) => {
            warn!("Unable to read config: {err}");
            ClientConfig::default()
        }
    }
}