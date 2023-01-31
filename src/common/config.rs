use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {



    #[serde(skip)] // generated from neighboring files
    pub profiles: Vec<Profile>,
}