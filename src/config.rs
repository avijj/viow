use rustyline;

use std::path::PathBuf;
use std::env::var;


#[derive(Debug)]
pub struct Config {
    config_dir: Option<PathBuf>,
    script_dir: Option<PathBuf>,
    plugin_dirs: Vec<PathBuf>,
    readline_config: rustyline::config::Config,
    readline_history: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let config_dir = Self::find_config_dir();
        let script_dir = Self::find_script_dir(&config_dir);
        let plugin_dirs = Self::find_plugins(&config_dir);
        let readline_config = Self::default_readline_config();
        let readline_history = Self::find_readline_history(&config_dir);

        Self {
            config_dir,
            script_dir,
            plugin_dirs,
            readline_config,
            readline_history,
        }
    }

    pub fn test_config() -> Self {
        let config_dir = Some(PathBuf::from("./"));
        let script_dir = Some(PathBuf::from("./"));
        let plugin_dirs = vec![];
        let readline_config = Self::default_readline_config();
        let readline_history = None;

        Self {
            config_dir,
            script_dir,
            plugin_dirs,
            readline_config,
            readline_history,
        }
    }

    fn find_config_dir() -> Option<PathBuf> {
        let mut path = PathBuf::new();

        if let Ok(viow_config_home) = var("VIOW_CONFIG_HOME") {
            path.push(&viow_config_home);
        } else if let Ok(xdg_config_home) = var("XDG_CONFIG_HOME") {
            if !xdg_config_home.is_empty() {
                path.push(xdg_config_home);
                path.push("viow");
            }
        } else if let Ok(home) = var("HOME") {
            if !home.is_empty() {
                path.push(home);
                path.push(".config");
                path.push("viow");
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

    fn find_plugins(config_dir: &Option<PathBuf>) -> Vec<PathBuf> {
        let mut rv = config_dir.as_ref()
            .map(|cfg| {
                let mut plugins = cfg.clone();
                plugins.push("plugins");
                plugins
            })
            .and_then(|cfg| std::fs::read_dir(cfg).ok())
            .map(|dir| {
                dir
                    .filter_map(|dir_entry| dir_entry.ok())
                    .map(|dir_entry| dir_entry.path())
                    .filter(|path| path.is_dir())
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec![]);

        rv.sort_unstable();
        rv
    }

    fn default_readline_config() -> rustyline::config::Config {
        rustyline::config::Builder::new()
            .max_history_size(1000)
            .history_ignore_dups(true)
            .auto_add_history(true)
            .tab_stop(4)
            .build()
    }

    fn find_readline_history(config_dir: &Option<PathBuf>) -> Option<PathBuf> {
        config_dir.as_ref().and_then(|cfg_dir| {
            let mut path = PathBuf::from(cfg_dir);
            path.push("history");

            if !path.exists() {
                // Try to create file. On failure go on without history.
                std::fs::File::create(&path)
                    .map_or(None, |_| Some(path))
            } else {
                Some(path)
            }
        })
    }

    pub fn get_script_dir(&self) -> Option<&PathBuf> {
        self.script_dir.as_ref()
    }

    pub fn get_config_dir(&self) -> Option<&PathBuf> {
        self.config_dir.as_ref()
    }

    pub fn get_plugin_dirs(&self) -> &[PathBuf] {
        &self.plugin_dirs
    }

    pub fn wave_cache_capacity(&self) -> usize {
        8
    }

    pub fn wave_cache_signals_per_tile(&self) -> usize {
        128
    }

    pub fn wave_cache_cycles_per_tile(&self) -> usize {
        1024
    }

    pub fn readline_config(&self) -> &rustyline::config::Config {
        &self.readline_config
    }

    pub fn readline_history(&self) -> Option<&PathBuf> {
        self.readline_history.as_ref()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::env::{set_var, remove_var};
    use std::fs::create_dir;
    use tempdir::TempDir;


    #[test]
    fn test_config_directories() {
        const DIRNAME: &'static str = "foo";

        // test init from VIOW_CONFIG_HOME
        {
            let tmpd = TempDir::new(DIRNAME).unwrap();
            let scriptd = tmpd.path().join("scripts");
            create_dir(&scriptd).unwrap();
            set_var("VIOW_CONFIG_HOME", tmpd.path());

            let config = Config::load();

            assert_eq!(tmpd.path(), *config.get_config_dir().unwrap());
            assert_eq!(scriptd, *config.get_script_dir().unwrap());

            remove_var("VIOW_CONFIG_HOME");
        }

        // test init from XDG_CONFIG_HOME
        {
            let tmpd = TempDir::new(DIRNAME).unwrap();
            let configd = tmpd.path().join("viow");
            let scriptd = configd.join("scripts");
            create_dir(&configd).unwrap();
            create_dir(&scriptd).unwrap();
            set_var("XDG_CONFIG_HOME", tmpd.path());

            let config = Config::load();

            assert_eq!(configd, *config.get_config_dir().unwrap());
            assert_eq!(scriptd, *config.get_script_dir().unwrap());

            remove_var("XDG_CONFIG_HOME");
        }

        // test init from HOME
        {
            let tmpd = TempDir::new(DIRNAME).unwrap();
            let viowd = tmpd.path().join(".config");
            let configd = viowd.join("viow");
            let scriptd = configd.join("scripts");
            create_dir(&viowd).unwrap();
            create_dir(&configd).unwrap();
            create_dir(&scriptd).unwrap();
            set_var("HOME", tmpd.path());

            let config = Config::load();

            assert_eq!(configd, *config.get_config_dir().unwrap());
            assert_eq!(scriptd, *config.get_script_dir().unwrap());

            remove_var("HOME");
        }

        // test non-existing script dir
        {
            let tmpd = TempDir::new(DIRNAME).unwrap();
            set_var("VIOW_CONFIG_HOME", tmpd.path());

            let config = Config::load();

            assert_eq!(tmpd.path(), *config.get_config_dir().unwrap());
            assert_eq!(None, config.get_script_dir());

            remove_var("VIOW_CONFIG_HOME");
        }

        // test non-existing config dir
        {
            let tmpd = TempDir::new(DIRNAME).unwrap();
            let configd = tmpd.path().join("viow_config");
            set_var("VIOW_CONFIG_HOME", configd);

            let config = Config::load();

            assert_eq!(None, config.get_config_dir());
            assert_eq!(None, config.get_script_dir());

            remove_var("VIOW_CONFIG_HOME");
        }
    }
}
