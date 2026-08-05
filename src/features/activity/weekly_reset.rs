use chrono::{Datelike, Duration, Local, Timelike, Weekday};
use sqlx::SqlitePool;
use std::time::Duration as StdDuration;
use tokio::time::sleep;

use crate::repository::users;

/// Запускает фоновую задачу еженедельного сброса шагов.
/// Каждый понедельник в 00:00 обнуляет `weeklysteps` и фиксирует лидера недели.
pub fn spawn(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            let next_monday = next_monday_midnight();
            let now = Local::now();
            let wait = (next_monday - now).to_std().unwrap_or(StdDuration::ZERO);

            tracing::info!("WeeklyReset -> scheduled for {next_monday}");
            sleep(wait).await;

            match users::reset_weekly_steps(&pool).await {
                Ok((leader_name, updated)) => {
                    tracing::info!("WeeklyReset -> Reset weeklySteps for {updated} users");
                    if let Some(name) = leader_name {
                        tracing::info!("WeeklyReset -> new leader: {name}");
                    }
                }
                Err(e) => tracing::error!("WeeklyReset -> error: {e}"),
            }
        }
    });
}

/// Ближайший понедельник в 00:00 (следующий, даже если сегодня понедельник).
fn next_monday_midnight() -> chrono::DateTime<Local> {
    let now = Local::now();
    let mut days_ahead = (Weekday::Mon.num_days_from_monday() as i64
        - now.weekday().num_days_from_monday() as i64)
        .rem_euclid(7);
    if days_ahead == 0 {
        days_ahead = 7;
    }

    let next = now + Duration::days(days_ahead);
    next.with_hour(0)
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap_or(next)
}
