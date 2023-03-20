use crate::common::timer::{PomodoroPeriod, PomodoroState, Timer, TimerGoal, TimerPeriod, TimerState};
use crate::common::ws_common::ServerToClient;
use crate::daemon::{SState, State};
use anyhow::{anyhow, bail, Result};
use chrono::{Duration,Utc};
use tokio::select;
use tracing::{error, info};
use crate::common::profile::Profile;

pub async fn create_timer(state: &SState, goal: TimerGoal, profile_name: String) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_some() {
        bail!("Timer is already created")
    }

    // find profile
    let profile = state.conf.read().await.profiles.iter().find(|p| p.name == profile_name).ok_or_else(|| anyhow!("Profile not found"))?.clone();

    let limit = calc_limit(&profile, &goal);

    let timer_state = TimerState {
        period: TimerPeriod::Paused { elapsed: Duration::zero(), limit },
        pomodoro: profile.pomodoro.as_ref().map(|_| PomodoroState {
            total_dur_worked: Duration::zero(),
            current_period: PomodoroPeriod::Work,
                small_breaks: 0,
            }),
    };

    *timer = Some(Timer { profile, goal, state: timer_state });
    info!("Timer created");

    drop(timer); // prevent deadlock
    unpause_timer(state).await?;

    Ok(ServerToClient::UpdateTimer(state.timer.lock().await.clone()))
}

pub async fn pause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };

    state.cancel_timer_tasks.notify_waiters();

    timer.state.period = match timer.state.period {
        TimerPeriod::Paused {..} => bail!("Timer is already paused"),
        TimerPeriod::Running { elapsed, start, limit } => {
            TimerPeriod::Paused {
                elapsed: elapsed + (Utc::now() - start),
                limit,
            }
        },
    };

    info!("Timer paused");
    Ok(ServerToClient::UpdateTimerState(timer.state.clone()))
}

pub async fn unpause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };

    timer.state.period = match timer.state.period {
        TimerPeriod::Running {..} => bail!("Timer is already running!"),
        TimerPeriod::Paused { elapsed, limit } => {

            if let Some(limit) = limit {
                // cancel any existing timer tasks
                state.cancel_timer_tasks.notify_waiters();

                // spawn new task
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = update_timer_task(state, (limit - elapsed).max(Duration::zero())).await {
                        error!("Error in timer task: {e}")
                    }
                });
            }

            TimerPeriod::Running {
                elapsed,
                start: Utc::now(),
                limit,
            }
        },
    };

    info!("Timer unpaused");
    Ok(ServerToClient::UpdateTimerState(timer.state.clone()))
}

async fn update_timer_task(state: SState, awake_in: Duration) -> Result<()> {
    info!("Waiting {awake_in:?}");
    select! {
        _ = state.cancel_timer_tasks.notified() => {return Ok(())}
        _ = tokio::time::sleep(awake_in.to_std()?) => { }
    }

    let mut mutex =  state.timer.lock().await;
    let Some(ref mut timer) =  *mutex else {
        bail!("Timer is not created")
    };
    if let TimerPeriod::Paused{..} = timer.state.period {
        bail!("Timer is paused")
    }

    match &mut timer.state.pomodoro {
        Some(ref mut _pomdoro) => {
            todo!()
        },
        None => {
            // it means that the duration ended so we can stop the timer
            // quick sanity check:
            let TimerGoal::Time{..} = timer.goal else { bail!("Reaching this state should be impossible"); };
            drop(mutex);
            state.ws_tx.send(stop_timer(&state).await?)?;
        }
    }

    Ok(())
}

pub async fn stop_timer(state: &SState) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_none() { bail!("Timer isn't created!") }
    state.cancel_timer_tasks.notify_waiters();
    *timer = None;

    info!("Timer stopped");
    Ok(ServerToClient::UpdateTimer(None))
}

fn calc_limit(profile: &Profile, goal: &TimerGoal) -> Option<Duration> {
    // pomodoro limit
    let mut limit = profile.pomodoro.as_ref().map(|p| p.work_dur);

    // time goal limit
    if let TimerGoal::Time (ref dur) = goal {
        if limit.is_none() || limit.unwrap() > *dur {
            limit = Some(*dur);
        }
    }
    limit
}