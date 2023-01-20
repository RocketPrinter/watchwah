use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;

#[derive(Serialize,Deserialize,Debug)]
pub enum ClientToServer {
    SetActiveProfile(Option<String>),

    RefreshConfig,

    Multiple(Vec<ClientToServer>),
}

#[derive(Serialize,Deserialize,Debug)]
pub enum ServerToClient {
    UpdateProfiles(Vec<String>),
    UpdateActiveProfile(Option<Profile>),

    RefreshedConfig,

    Multiple(Vec<ServerToClient>),
}

// todo: todo system