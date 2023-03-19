use crate::common::timer::{PomodoroPeriod, PomodoroState, Timer, TimerGoal, TimerPeriod, TimerState};
use crate::common::ws_common::ServerToClient;
use crate::daemon::{SState, State};
use anyhow::{anyhow, bail, Result};
use chrono::{Duration,Utc};
use tokio::select;
use tracing::{error, info};

pub async fn create_timer(state: &SState, goal: TimerGoal, profile_name: String) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_some() {
        bail!("Timer is already created")
    }

    let profile = state.conf.read().await.profiles.iter().find(|p| p.name == profile_name).ok_or_else(|| anyhow!("Profile not found"))?.clone();

    // set the time left to the pomodoro work period, or never
    let mut starting_dur = profile.pomodoro.as_ref().map(|p| p.work_dur);

    // if the goal is time-based, make sure we don't overshoot the time left
    if let TimerGoal::Time (ref dur) = goal {
        starting_dur = min_option_dur(starting_dur, Some(*dur));
    }

    let timer_state = TimerState {
        period: TimerPeriod::Paused { total: starting_dur, dur_left: starting_dur },
        pomodoro: profile.pomodoro.as_ref().map(|_| PomodoroState {
            total_dur_worked: Duration::zero(),
            current_period: PomodoroPeriod::Work,
                small_breaks: 0,
            }),
    };

    *timer = Some(Timer { profile, goal, state: timer_state });
    info!("Timer created");

    drop(timer);
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
        TimerPeriod::Running { total, end } => {
            TimerPeriod::Paused {
                total,
                dur_left: end.map(|end| end - Utc::now() ),
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
        TimerPeriod::Paused { total, dur_left } => {

            let end = dur_left.and_then(|dur_left|  Utc::now().checked_add_signed(dur_left) );

            if let Some(dur_left) = dur_left {
                // cancel any existing timer tasks
                state.cancel_timer_tasks.notify_waiters();
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = update_timer_task(state, dur_left).await {
                        error!("Error in timer task: {e}")
                    }
                });
            }

            TimerPeriod::Running {
                total,
                end,
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

fn min_option_dur(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}