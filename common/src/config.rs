use std::path::{Path, PathBuf};

pub fn get_config_path() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap())
        .join(".config")
        .join("watchwah")
}