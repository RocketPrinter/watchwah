use crate::common::ws_common::ServerToClient;
use crate::daemon::{timer_service, SState};
use anyhow::{anyhow, Result};

pub async fn set_active_profile(state: &SState, name: Option<String>) -> Result<ServerToClient> {
    let mut profile = state.active_profile.write().await;
    match name {
        Some(name) => {
            let conf = state.conf.read().await;
            let new_profile = conf
                .profiles
                .iter()
                .find(|p| p.name == name)
                .cloned()
                .ok_or(anyhow!("Cannot find profile!"))?;
            *profile = Some(new_profile);
        }
        None => *profile = None,
    }
    
    Ok(ServerToClient::Multiple(vec![
        timer_service::stop_timer(state).await?,
        active_profile_msg(state).await,
    ]))
}

pub async fn active_profile_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateActiveProfile(state.active_profile.read().await.clone())
}

pub async fn profiles_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateProfiles(
        state
            .conf
            .read()
            .await
            .profiles
            .iter()
            .map(|p| p.name.to_string())
            .collect(),
    )
}
