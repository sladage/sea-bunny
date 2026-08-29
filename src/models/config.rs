use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use url::Url;

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(Config::default()));

pub fn config() -> Config {
    CONFIG.lock().unwrap().clone()
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(skip)]
    _path: PathBuf,
    pub nextcloud_server: Option<Url>,
}

impl Config {
    fn load_toml(path: &Path) -> Result<Config> {
        let mut file = File::open(path)?;
        let mut toml = String::new();
        file.read_to_string(&mut toml)?;
        let mut cfg: Config = toml::from_str(&toml)?;
        cfg._path = path.to_path_buf();
        Ok(cfg)
    }

    fn save_toml(&self, path: &Path) -> Result<()> {
        let toml = toml::to_string(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut Config),
    {
        let mut cfg = self.clone();
        f(&mut cfg);
        cfg.save_toml(&self._path).expect("Unable to save config.");
        *CONFIG.lock().unwrap() = cfg;
    }

    pub fn init() {
        let bsd = directories::BaseDirs::new().expect("Unable to get base dirs.");
        let path = bsd.config_dir().join("sea-bunny/config.toml");
        let cfg = if !path.exists() {
            // create default config file
            let default_cfg = Config {
                _path: path.clone(),
                nextcloud_server: None,
            };
            default_cfg
                .save_toml(&path)
                .expect("Unable to save default config.");
            default_cfg
        } else {
            Self::load_toml(&path).expect("Unable to load config.")
        };
        *CONFIG.lock().unwrap() = cfg;
    }
}
