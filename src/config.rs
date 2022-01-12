use crate::error::*;

use tui::style::*;
use std::path::{Path,PathBuf};
use std::env::var;


#[derive(Debug)]
pub struct Config {
    config_dir: Option<PathBuf>,
    script_dir: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let config_dir = Self::find_config_dir();
        let script_dir = Self::find_script_dir(&config_dir);

        Self {
            config_dir,
            script_dir,
        }
    }

    fn find_config_dir() -> Option<PathBuf> {
        let mut path = PathBuf::new();

        if let Ok(twv_config_home) = var("TWV_CONFIG_HOME") {
            path.push(&twv_config_home);
        } else if let Ok(xdg_config_home) = var("XDG_CONFIG_HOME") {
            if !xdg_config_home.is_empty() {
                path.push(xdg_config_home);
                path.push("twv");
            }
        } else if let Ok(home) = var("HOME") {
            if !home.is_empty() {
                path.push(home);
                path.push(".config");
                path.push("twv");
            }
        }

        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    fn find_script_dir(config_dir: &Option<PathBuf>) -> Option<PathBuf> {
        if let Some(config_dir) = config_dir {
            let mut path = PathBuf::from(config_dir);
            path.push("scripts");

            if path.exists() {
                Some(path)
            } else {
                None
            }
        } else {
            None
        }
    }


    pub fn get_script_dir(&self) -> Option<&PathBuf> {
        self.script_dir.as_ref()
    }
}


