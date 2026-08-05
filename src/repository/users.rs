use sqlx::{FromRow, SqlitePool};

use crate::features::activity::model::UserActivity;

/// Запись таблицы `users`.
#[derive(Debug, FromRow)]
pub struct User {
    pub password: String,
    pub steps: i64,
    pub weekly_steps: i64,
}

pub async fn get_user(pool: &SqlitePool, login: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT password, steps, weeklysteps AS weekly_steps
         FROM users
         WHERE login = ?",
    )
    .bind(login)
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &SqlitePool, login: &str, password: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (login, password, steps, weeklysteps) VALUES (?, ?, 0, 0)")
        .bind(login)
        .bind(password)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn change_login(
    pool: &SqlitePool,
    old_login: &str,
    new_login: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET login = ? WHERE login = ?")
        .bind(new_login)
        .bind(old_login)
        .execute(pool)
        .await?;
    Ok(())
}

/// Обновляет шаги пользователя и возвращает полный список пользователей,
/// отсортированный по `weeklysteps` по убыванию.
pub async fn update_steps_and_list(
    pool: &SqlitePool,
    activity: &UserActivity,
) -> Result<Vec<UserActivity>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE users SET steps = ?, weeklysteps = ? WHERE login = ?")
        .bind(activity.steps)
        .bind(activity.weekly_steps)
        .bind(&activity.login)
        .execute(&mut *tx)
        .await?;

    let rows = sqlx::query_as::<_, UserActivity>(
        "SELECT login, steps, weeklysteps AS weekly_steps
         FROM users
         ORDER BY weeklysteps DESC",
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(rows)
}

/// Еженедельный сброс: определяет лидера (максимум `weeklysteps`), сохраняет его
/// в `currentleader` и обнуляет `weeklysteps` у всех пользователей.
/// Возвращает (логин лидера, количество обновлённых пользователей).
pub async fn reset_weekly_steps(pool: &SqlitePool) -> Result<(Option<String>, u64), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let leader_name: Option<String> = sqlx::query_scalar(
        "SELECT login
         FROM users
         WHERE weeklysteps > 0
         ORDER BY weeklysteps DESC
         LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(ref name) = leader_name {
        sqlx::query("DELETE FROM currentleader")
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO currentleader (currentleader) VALUES (?)")
            .bind(name)
            .execute(&mut *tx)
            .await?;
    }

    let result = sqlx::query("UPDATE users SET weeklysteps = 0 WHERE weeklysteps != 0")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok((leader_name, result.rows_affected()))
}
