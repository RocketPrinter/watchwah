use crate::common::timer::{PomodoroPeriod, PomodoroState, Timer, TimerGoal, TimerPeriod, TimerState};
use crate::common::ws_common::ServerToClient;
use crate::daemon::SState;
use anyhow::{anyhow, bail, Result};
use chrono::Utc;

pub async fn create_timer(state: &SState, mut goal: TimerGoal, profile_name: String) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_some() {
        bail!("Timer is already created")
    }

    let profile = state.conf.read().await.profiles.iter().find(|p| p.name == profile_name).ok_or_else(|| anyhow!("Profile not found"))?.clone();

    // set the time left to the pomodoro work period. or never
    let mut dur_left = profile.pomodoro.as_ref().map(|p| p.work_dur);
    // if the goal is time-based, make sure we don't overshoot the time left
    if let TimerGoal::Time { ref mut left, ..} = goal {
        if dur_left.is_none() || dur_left.unwrap() > *left {
            dur_left = Some(*left);
        }
        // subtract the time left from the goal
        *left -= dur_left.unwrap();
    }

    let state = TimerState {
        period: TimerPeriod::Paused { dur_left },
        goal,
        pomodoro: profile.pomodoro.as_ref().map(|_| PomodoroState {
                current_period: PomodoroPeriod::Work,
                small_breaks: 0,
            }),
    };

    *timer = Some(Timer { profile, state });

    todo!()
}

pub async fn pause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };

    state.cancel_timer_tasks.notify_waiters();

    timer.state.period = match timer.state.period {
        TimerPeriod::Paused {..} => bail!("Timer is already paused"),
        TimerPeriod::Running { end, .. } => {
            TimerPeriod::Paused {
                dur_left: end.and_then(|end| (end - Utc::now()).to_std().ok() ),
            }
        },
    };

    Ok(ServerToClient::UpdateTimerState(timer.state.clone()))
}

pub async fn unpause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };

    timer.state.period = match timer.state.period {
        TimerPeriod::Running {..} => bail!("Timer is already running!"),
        TimerPeriod::Paused { dur_left } => {
            // todo: this is a bit ugly
            let dur_left = dur_left.and_then(|dur_left| chrono::Duration::from_std(dur_left).ok() );
            let now = Utc::now();
            let end = dur_left.and_then(|dur_left| now.checked_add_signed(dur_left) );

            // todo: start task here

            TimerPeriod::Running {
                start: now,
                end,
            }
        },
    };

    Ok(ServerToClient::UpdateTimerState(timer.state.clone()))
}

pub async fn stop_timer(state: &SState) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_none() { bail!("Timer isn't created!") }
    state.cancel_timer_tasks.notify_waiters();
    *timer = None;

    Ok(timer_msg(state).await)
}

pub async fn timer_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateTimer(state.timer.lock().await.clone())
}