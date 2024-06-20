use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub id: String,
    pub instance: String,
    pub sn_instance: String,
    pub sn_username: String,
    pub sn_password: String,
    pub bin: String,
}

impl Config {
    pub fn from_toml_file() -> io::Result<Self> {
        let path = get_config_dir().join("config.toml");
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents).unwrap_or_default();
        Ok(config)
    }

    pub fn to_toml_file(&self) -> io::Result<()> {
        let path = get_config_dir().join("config.toml");
        let toml_string = toml::to_string(self).unwrap_or_default();
        let mut file = File::create(path)?;
        file.write_all(toml_string.as_bytes())?;
        Ok(())
    }
}

pub fn get_config_dir() -> PathBuf {
    let config_dir = config_dir().expect("Unable to get config directory");
    PathBuf::from(format!("{}/elasticnow", config_dir.display()))
}

pub fn make_dir_if_none() {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        if std::fs::create_dir_all(&config_dir).is_err() {
            tracing::error!("Unable to create config directory {:?}", config_dir);
        }
    }
}
