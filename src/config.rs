use crate::utils::BoxedError;
use config::Config;
use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir, write};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub(crate) moco_company: Option<String>,
    pub(crate) moco_api_key: Option<String>,
    pub(crate) moco_bot_api_key: Option<String>,
    pub(crate) moco_user_id: Option<i64>,
}

fn get_config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|dir| dir.join("mococli").join("mococli.json"))
}

pub(crate) fn init() -> Result<AppConfig, BoxedError> {
    let config_file = get_config_path();
    let config_file = match config_file {
        Some(path) => {
            if !path.exists() {
                if !&path.parent().unwrap().exists() {
                    create_dir(path.parent().unwrap())?;
                }
                File::create(&path)?;
                write(&path, "{}")?;
            }
            path
        }
        None => panic!("Can't find os config directory"),
    };
    Ok(Config::builder()
        .add_source(config::File::from(config_file))
        .build()?
        .try_deserialize::<AppConfig>()?)
}

impl AppConfig {
    pub(crate) fn write_config(&self) -> Result<(), BoxedError> {
        let config_file = get_config_path();
        match config_file {
            Some(file) => {
                let json_string = serde_json::to_string(self)?;
                write(file, json_string)?;
            }
            None => panic!("Can't find os config directory"),
        };
        Ok(())
    }
}
