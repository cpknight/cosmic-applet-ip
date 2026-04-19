// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

/// Which interface the user wants displayed.
///
/// `Auto` means "whichever interface owns the current default route".
/// `Pinned(name)` means "always display the IP of this interface even if it's down".
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Selection {
    Auto,
    Pinned(String),
}

impl Default for Selection {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Eq, PartialEq, CosmicConfigEntry, Serialize, Deserialize)]
#[version = 1]
pub struct IpAppletConfig {
    pub selection: Selection,
    /// Show the IPv4 text label next to the icon in the panel (default: true).
    pub show_label: bool,
}

impl Default for IpAppletConfig {
    fn default() -> Self {
        Self {
            selection: Selection::Auto,
            show_label: true,
        }
    }
}

impl IpAppletConfig {
    pub fn load() -> Self {
        match cosmic_config::Config::new(crate::app::APP_ID, Self::VERSION) {
            Ok(ctx) => match Self::get_entry(&ctx) {
                Ok(cfg) => cfg,
                Err((errs, cfg)) => {
                    for err in errs {
                        tracing::warn!(?err, "error loading IP applet config");
                    }
                    cfg
                }
            },
            Err(err) => {
                tracing::warn!(?err, "could not open cosmic config, using defaults");
                Self::default()
            }
        }
    }
}
