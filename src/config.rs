
use std::path::PathBuf;
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

    pub fn test_config() -> Self {
        let config_dir = Some(PathBuf::from("./"));
        let script_dir = Some(PathBuf::from("./"));

        Self {
            config_dir,
            script_dir,
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


    pub fn get_script_dir(&self) -> Option<&PathBuf> {
        self.script_dir.as_ref()
    }

    pub fn get_config_dir(&self) -> Option<&PathBuf> {
        self.config_dir.as_ref()
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
