use std::cmp::Ordering;
use crate::SState;
use anyhow::{anyhow, bail, Result};
use chrono::{Duration, Utc};
use common::timer::{PeriodProgress, PeriodType, Timer, TimerGoal, TimerState};
use common::ws_common::ServerToClient;
use tokio::select;
use tracing::{debug, error, info};

pub async fn create_timer(
    timer: &mut Option<Timer>,
    state: &SState,
    goal: TimerGoal,
    profile_name: String,
    start_in: Option<Duration>,
) -> Result<SyncToken> {

    if timer.is_some() {
        bail!("Timer is already created")
    }

    // find profile
    let profile = state
        .conf
        .read()
        .await
        .profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| anyhow!("Profile not found"))?
        .clone();

    // create new timer
    *timer = Some(Timer {
        profile,
        goal,
        state: TimerState {
            progress: PeriodProgress::Uninit,
            period: PeriodType::Uninit,
            total_dur_worked: Duration::zero(),
            small_breaks: 0,
        },
    });

    let timer = timer.as_mut().unwrap();

    _ = if let Some(start_in) = start_in {
        // start with a break
        set_next_period(timer, state.clone(), (PeriodType::Starting, Some(start_in)))?
    } else {
        // start normally
        set_next_period(timer, state.clone(), pick_next_period(timer))?
    };
    info!("Timer created");
    Ok(SyncToken::Timer)
}

pub fn pause_timer(timer: &mut Timer, state: &SState) -> Result<SyncToken> {
    state.cancel_timer_task.notify_waiters();

    timer.state.progress = match timer.state.progress {
        PeriodProgress::Uninit => bail!("Timer is not initialized"),
        PeriodProgress::Paused { .. } => bail!("Timer is already paused"),
        PeriodProgress::Running {
            elapsed,
            start,
            limit,
        } => PeriodProgress::Paused {
            elapsed: elapsed + (Utc::now() - start),
            limit,
        },
    };

    info!("Timer paused");
    Ok(SyncToken::TimerState)
}

pub fn unpause_timer(timer: &mut Timer, state: &SState) -> Result<SyncToken> {
    timer.state.progress = match timer.state.progress {
        PeriodProgress::Uninit => bail!("Timer is not initialized"),
        PeriodProgress::Running { .. } => bail!("Timer is already running!"),
        PeriodProgress::Paused { elapsed, limit } => {
            if let Some(limit) = limit {
                let awake_in = (limit - elapsed).max(Duration::zero());
                spawn_task(state.clone(), awake_in);
            }

            PeriodProgress::Running {
                elapsed,
                start: Utc::now(),
                limit,
            }
        }
    };

    info!("Timer unpaused");
    Ok(SyncToken::TimerState)
}

pub fn skip_period(timer: &mut Timer, state: &SState) -> Result<SyncToken> {
    let msg = set_next_period(timer, state.clone(), pick_next_period(timer))?;

    info!("Timer skipped period");
    Ok(msg)
}

pub fn stop_timer(timer: &mut Option<Timer>, state: &SState) -> Result<SyncToken> {
    if timer.is_none() {
        bail!("Timer isn't created!")
    }
    state.cancel_timer_task.notify_waiters();
    *timer = None;

    info!("Timer stopped");
    Ok(SyncToken::Timer)
}

// region Helpers

/// Used to determine what ServerToClient message to send to clients to share the timer state
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncToken {
    #[allow(dead_code)]
    None,
    TimerState,
    Timer
}

impl SyncToken {
    pub fn to_msg(self, timer: Option<&Timer>) -> Option<ServerToClient> {
        Some(match self {
            SyncToken::None => return None,
            SyncToken::TimerState => {
                let Some(timer) = timer else {return None};
                ServerToClient::UpdateTimerState(Box::new(timer.state.clone()))
            },
            SyncToken::Timer => ServerToClient::UpdateTimer(timer.map(|t| Box::new(t.clone()))),
        })
    }
}

fn pick_next_period(timer: &Timer) -> (PeriodType, Option<Duration>) {
    use PeriodType::*;
    let time_left = TimerGoal::time_left(timer);
    if time_left == Some(Duration::zero()){
        // we stop the timer by sending in a zero duration period
        (Uninit, Some(Duration::zero()))
    } else if let Some(pomodoro) = &timer.profile.pomodoro {
        // pomodoro logic
        match timer.state.period {
            Work => {
                if timer.state.small_breaks >= pomodoro.small_breaks_before_big_one {
                    (LongBreak, Some(pomodoro.long_break_dur))
                } else {
                    (ShortBreak, Some(pomodoro.short_break_dur))
                }
            }
            _ => (Work, Some(match time_left {
                Some(time_left) => time_left.min(pomodoro.work_dur),
                None => pomodoro.work_dur,
            })),
        }
    } else {
        // normal logic
        (Work, time_left )
    }
}

fn set_next_period(timer: &mut Timer, state: SState, period: (PeriodType, Option<Duration>)) -> Result<SyncToken> {

    // setup next period
    timer.state.period = period.0;
    timer.state.progress = PeriodProgress::Running {
        elapsed: Duration::zero(),
        start: Utc::now(),
        limit: period.1,
    };

    // extra logic per period type
    match timer.state.period {
        PeriodType::Work =>
            if let Some(dur) = period.1 {
                timer.state.total_dur_worked = timer.state.total_dur_worked + dur;
            }
        PeriodType::ShortBreak => timer.state.small_breaks += 1,
        PeriodType::LongBreak => timer.state.small_breaks = 0,
        _ => {}
    };

    // make sure there are no other tasks
    state.cancel_timer_task.notify_waiters();

    // spawn timer task
    if let Some(dur) = period.1 {
        spawn_task(state, dur);
    }

    info!("Next period: {period:?}");
    Ok(SyncToken::TimerState)
}

fn spawn_task(state: SState, awake_in: Duration) {
    // cancel any existing timer tasks
    state.cancel_timer_task.notify_waiters();

    // spawn task
    tokio::spawn(async move {
        if let Err(e) = timer_task(state, awake_in).await {
            error!("Error in timer task: {e}")
        }
    });

    async fn timer_task(state: SState, awake_in: Duration) -> Result<()> {
        if awake_in <= Duration::zero() {
            // stop the timer
            // todo: hacky solution, far from ideal
            if let Some(msg) = stop_timer(&mut *state.timer.lock().await, &state)?.to_msg(None) {
                state.ws_tx.send(msg).unwrap();
            }
            return Ok(());
        }

        // await for the task to cet cancelled or the duration to pass
        select! {
                _ = state.cancel_timer_task.notified() => return Ok(()),
                _ = tokio::time::sleep(awake_in.to_std()?) => { }
        }

        // start next period
        let mut timer = state.timer.lock().await;
        let timer = timer.as_mut().ok_or_else(|| anyhow!("Timer isn't created!"))?;
        let msg = set_next_period(timer, state.clone(), pick_next_period(timer))?.to_msg(Some(timer));

        // sync clients if necessary
        if let Some(msg) = msg {
            state.ws_tx.send(msg)?;
        }

        Ok(())
    }
}
// endregion