use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    SetActiveProfile(Option<String>),

    Multiple(Vec<ClientToServer>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    UpdateProfiles(Vec<String>),
    UpdateActiveProfile(Option<Profile>),

    RefreshedConfig,

    Multiple(Vec<ServerToClient>),
}

// todo: todo system