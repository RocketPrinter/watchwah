use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;
use anyhow::{anyhow, Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {



    #[serde(skip)] // generated from neighboring files
    pub profiles: Vec<Profile>,
}

pub fn get_config_path() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap()).join(".config").join("watchwah")
}

pub fn load(path: PathBuf) ->  Result<Config> {
    let mut conf: Option<Config> = None;
    let mut profiles: Vec<Profile> = vec![];

    for file in fs::read_dir(path)?.filter_map(|f|f.ok()) {
        if !file.path().ends_with(".toml") {continue}

        let contents = fs::read_to_string(file.path())?;

        if file.file_name() == "config.toml" {
            // main config
            conf = Some(toml::from_str(&contents)?);
        } else {
            let mut profile = toml::from_str::<Profile>(&contents)?;
            profile.name = file.path().file_stem().unwrap().to_str().unwrap().to_string();
            profiles.push(profile);
        }
    }

    if let Some(mut conf) = conf {
        conf.profiles = profiles;
        Ok(conf)
    } else {
        Err(anyhow!("config.toml missing!"))
    }
}