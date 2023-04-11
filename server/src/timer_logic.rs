use common::timer::{
    PomodoroPeriod, PomodoroState, Timer, TimerGoal, TimerPeriod, TimerState,
};
use common::ws_common::ServerToClient;
use crate::{SState};
use anyhow::{anyhow, bail, Result};
use chrono::{Duration, Utc};
use tokio::select;
use tracing::{error, info};

pub async fn create_timer(
    state: &SState,
    goal: TimerGoal,
    profile_name: String,
) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
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

    // we set the limit later cause we need the object to be created first
    let timer_state = TimerState {
        period: TimerPeriod::Running {
            elapsed: Duration::zero(),
            start: Default::default(),
            limit: None,
        },
        pomodoro: profile.pomodoro.as_ref().map(|_| PomodoroState {
            total_dur_worked: Duration::zero(),
            current_period: PomodoroPeriod::Work,
            small_breaks: 0,
        }),
    };

    let mut new_timer = Timer {
        profile,
        goal,
        state: timer_state,
    };

    // now that the object is created we can use the get_limit function
    let limit = calc_work_period_limit(&new_timer);

    new_timer.state.period = TimerPeriod::Running {
        elapsed: Duration::zero(),
        start: Utc::now(),
        limit,
    };
    *timer = Some(new_timer);

    if let Some(limit) = limit {
        spawn_task(state.clone(), limit);
    }

    info!("Timer started");

    Ok(ServerToClient::UpdateTimer(
        timer.clone().map(Box::new)),
    )
}

pub async fn pause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };

    state.cancel_timer_tasks.notify_waiters();

    timer.state.period = match timer.state.period {
        TimerPeriod::Paused { .. } => bail!("Timer is already paused"),
        TimerPeriod::Running {
            elapsed,
            start,
            limit,
        } => TimerPeriod::Paused {
            elapsed: elapsed + (Utc::now() - start),
            limit,
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
        TimerPeriod::Running { .. } => bail!("Timer is already running!"),
        TimerPeriod::Paused { elapsed, limit } => {
            if let Some(limit) = limit {
                let awake_in = (limit - elapsed).max(Duration::zero());
                spawn_task(state.clone(), awake_in);
            }

            TimerPeriod::Running {
                elapsed,
                start: Utc::now(),
                limit,
            }
        }
    };

    info!("Timer unpaused");
    Ok(ServerToClient::UpdateTimerState(timer.state.clone()))
}

pub async fn stop_timer(state: &SState) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_none() {
        bail!("Timer isn't created!")
    }
    state.cancel_timer_tasks.notify_waiters();
    *timer = None;

    info!("Timer stopped");
    Ok(ServerToClient::UpdateTimer(None))
}

fn spawn_task(state: SState, awake_in: Duration) {
    if awake_in <= Duration::zero() { return; }

    // cancel any existing timer tasks
    state.cancel_timer_tasks.notify_waiters();

    // spawn task
    tokio::spawn(async move {
        if let Err(e) = timer_task(state, awake_in).await {
            error!("Error in timer task: {e}")
        }
    });
}


async fn timer_task(state: SState, awake_in: Duration) -> Result<()> {
    info!("Waiting {awake_in:?}");
    // we await the period or cancel if notified
    select! {
        _ = state.cancel_timer_tasks.notified() => {return Ok(())}
        _ = tokio::time::sleep(awake_in.to_std()?) => { }
    }

    let mut mutex = state.timer.lock().await;
    let Some(ref mut timer) =  *mutex else {
        bail!("Timer is not created")
    };
    if let TimerPeriod::Paused { .. } = timer.state.period {
        bail!("Timer is paused")
    }

    match &mut timer.state.pomodoro {
        Some(ref mut pomodoro) => {
            let pomo_settings = timer.profile.pomodoro.as_ref().unwrap();

            let limit = match pomodoro.current_period {
                PomodoroPeriod::Work => {
                    // we did some work, break comes next
                    // but first we add the duration worked to the total
                    pomodoro.total_dur_worked =
                        pomodoro.total_dur_worked + timer.state.period.elapsed();

                    // check if we finished
                    if let TimerGoal::Time(limit) = &timer.goal {
                        if pomodoro.total_dur_worked >= *limit {
                            // we reached the goal, stop the timer
                            drop(mutex);
                            state.ws_tx.send(stop_timer(&state).await?)?;
                            return Ok(());
                        }
                    }

                    if pomodoro.small_breaks >= pomo_settings.small_breaks_between_big_ones {
                        // we took enough small breaks, time for a big one
                        pomodoro.small_breaks = 0;
                        pomodoro.current_period = PomodoroPeriod::LongBreak;
                        Some(pomo_settings.long_break_dur)
                    } else {
                        // small break
                        pomodoro.small_breaks += 1;
                        pomodoro.current_period = PomodoroPeriod::ShortBreak;
                        Some(pomo_settings.small_break_dur)
                    }
                }
                PomodoroPeriod::ShortBreak | PomodoroPeriod::LongBreak => {
                    // we took a break, work follows next

                    pomodoro.current_period = PomodoroPeriod::Work;
                    calc_work_period_limit(timer)
                }
            };

            // start the period again
            timer.state.period = TimerPeriod::Running {
                elapsed: Duration::zero(),
                start: Utc::now(),
                limit,
            };

            if let Some(limit) = limit {
                spawn_task(state.clone(), limit);
            }

            state.ws_tx.send(ServerToClient::UpdateTimerState(timer.state.clone()))?;
        }
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

// how long the timer should last for the next work period
fn calc_work_period_limit(timer: &Timer) -> Option<Duration> {
    // pomodoro limit
    let mut limit = timer.profile.pomodoro.as_ref().map(|p| p.work_dur);

    // time goal limit
    if let TimerGoal::Time(ref dur) = timer.goal {
        let total_dur_worked = timer
            .state
            .pomodoro
            .as_ref()
            .map(|p| p.total_dur_worked)
            .unwrap_or(Duration::zero());
        let dur = *dur - total_dur_worked;
        if limit.is_none() || limit.unwrap() > dur {
            limit = Some(dur);
        }
    }

    limit
}