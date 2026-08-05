use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rust_friendsactivity_backend::repository::users;
use rust_friendsactivity_backend::router::build_router;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

/// Создаёт роутер и пул на in-memory SQLite с применёнными миграциями.
async fn setup() -> (Router, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("подключиться к in-memory SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("применить миграции");

    let router = build_router(pool.clone());
    (router, pool)
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, String) {
    let res = app.clone().oneshot(req).await.expect("выполнить запрос");
    let status = res.status();
    let bytes = res
        .into_body()
        .collect()
        .await
        .expect("прочитать тело")
        .to_bytes();
    let text = String::from_utf8(bytes.to_vec()).expect("тело в UTF-8");
    (status, text)
}

async fn post_json(app: &Router, uri: &str, body: Value) -> (StatusCode, String) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("собрать запрос");
    send(app, req).await
}

async fn get(app: &Router, uri: &str) -> (StatusCode, String) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("собрать запрос");
    send(app, req).await
}

#[tokio::test]
async fn register_success_returns_token() {
    let (app, _pool) = setup().await;

    let (status, body) = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert!(value["token"].is_string());
}

#[tokio::test]
async fn register_conflict_returns_409_text() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let (status, body) = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "other"}),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, "User is already exists");
}

#[tokio::test]
async fn login_success_returns_token() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let (status, body) = post_json(
        &app,
        "/login",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert!(value["token"].is_string());
}

#[tokio::test]
async fn login_wrong_password_returns_400_text() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let (status, body) = post_json(
        &app,
        "/login",
        json!({"login": "alice", "password": "wrong"}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "Password is incorrect");
}

#[tokio::test]
async fn login_unknown_user_returns_400_text() {
    let (app, _pool) = setup().await;

    let (status, body) = post_json(
        &app,
        "/login",
        json!({"login": "ghost", "password": "secret"}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "User does not exists");
}

#[tokio::test]
async fn login_does_not_persist_token() {
    let (app, pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/login",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/login",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;

    // В Ktor токен сохраняется только при регистрации, при логине — нет.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tokens")
        .fetch_one(&pool)
        .await
        .expect("запрос к tokens");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn login_update_success_changes_login() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let (status, body) = post_json(
        &app,
        "/login_update",
        json!({"login": "alice", "newLogin": "alice_new"}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert_eq!(value["message"], "ok!");

    // Под новым логином вход работает.
    let (status, _) = post_json(
        &app,
        "/login",
        json!({"login": "alice_new", "password": "secret"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn login_update_unknown_user_returns_400_text() {
    let (app, _pool) = setup().await;

    let (status, body) = post_json(
        &app,
        "/login_update",
        json!({"login": "ghost", "newLogin": "ghost_new"}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "User does not exists");
}

#[tokio::test]
async fn login_update_to_taken_login_returns_400_text() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/register",
        json!({"login": "bob", "password": "secret"}),
    )
    .await;
    let (status, body) = post_json(
        &app,
        "/login_update",
        json!({"login": "alice", "newLogin": "bob"}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "error due to nick changing");
}

#[tokio::test]
async fn post_activity_returns_sorted_friends_list() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/register",
        json!({"login": "bob", "password": "secret"}),
    )
    .await;

    let (status, body) = post_json(
        &app,
        "/post_activity",
        json!({"login": "alice", "steps": 100, "weeklySteps": 30}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert_eq!(value["errorMessage"], Value::Null);
    assert_eq!(value["leader"], Value::Null);

    let list = value["friendsList"]
        .as_array()
        .expect("friendsList — массив");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["login"], "alice");
    assert_eq!(list[0]["weeklySteps"], 30);
    assert_eq!(list[1]["weeklySteps"], 0);

    // Сортировка по убыванию weeklySteps.
    let (_, body) = post_json(
        &app,
        "/post_activity",
        json!({"login": "bob", "steps": 200, "weeklySteps": 90}),
    )
    .await;
    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    let list = value["friendsList"]
        .as_array()
        .expect("friendsList — массив");
    assert_eq!(list[0]["login"], "bob");
    assert_eq!(list[1]["login"], "alice");
}

#[tokio::test]
async fn post_activity_unknown_user_returns_400_text() {
    let (app, _pool) = setup().await;

    let (status, body) = post_json(
        &app,
        "/post_activity",
        json!({"login": "ghost", "steps": 1, "weeklySteps": 1}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, "User does not exists, so cant fetch data");
}

#[tokio::test]
async fn post_activity_field_order_matches_ktor() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let (_, body) = post_json(
        &app,
        "/post_activity",
        json!({"login": "alice", "steps": 1, "weeklySteps": 2}),
    )
    .await;

    let i1 = body.find("friendsList").expect("friendsList в ответе");
    let i2 = body.find("errorMessage").expect("errorMessage в ответе");
    let i3 = body.find("leader").expect("leader в ответе");
    assert!(
        i1 < i2 && i2 < i3,
        "порядок полей должен совпадать с Ktor: {body}"
    );
}

#[tokio::test]
async fn fetch_returns_user_data() {
    let (app, _pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/post_activity",
        json!({"login": "alice", "steps": 123, "weeklySteps": 45}),
    )
    .await;

    let (status, body) = get(&app, "/fetch/alice").await;
    assert_eq!(status, StatusCode::OK);

    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert_eq!(value["steps"], 123);
    assert_eq!(value["weeklySteps"], 45);
    assert_eq!(value["errorMessage"], Value::Null);
}

#[tokio::test]
async fn fetch_unknown_user_returns_200_with_error_message() {
    let (app, _pool) = setup().await;

    let (status, body) = get(&app, "/fetch/ghost").await;
    assert_eq!(status, StatusCode::OK);

    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert_eq!(value["steps"], Value::Null);
    assert_eq!(value["weeklySteps"], Value::Null);
    assert_eq!(value["errorMessage"], "User not found");
}

#[tokio::test]
async fn weekly_reset_picks_leader_and_zeroes_weekly_steps() {
    let (app, pool) = setup().await;

    let _ = post_json(
        &app,
        "/register",
        json!({"login": "alice", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/register",
        json!({"login": "bob", "password": "secret"}),
    )
    .await;
    let _ = post_json(
        &app,
        "/post_activity",
        json!({"login": "alice", "steps": 100, "weeklySteps": 50}),
    )
    .await;
    let _ = post_json(
        &app,
        "/post_activity",
        json!({"login": "bob", "steps": 200, "weeklySteps": 90}),
    )
    .await;

    let (leader, updated) = users::reset_weekly_steps(&pool)
        .await
        .expect("сбросить шаги");
    assert_eq!(leader.as_deref(), Some("bob"));
    assert_eq!(updated, 2);

    // После сброса в ответе post_activity появляется лидер, а weeklySteps обнулены.
    let (_, body) = post_json(
        &app,
        "/post_activity",
        json!({"login": "alice", "steps": 0, "weeklySteps": 0}),
    )
    .await;
    let value: Value = serde_json::from_str(&body).expect("JSON в ответе");
    assert_eq!(value["leader"], "bob");
    let list = value["friendsList"]
        .as_array()
        .expect("friendsList — массив");
    assert!(list.iter().all(|u| u["weeklySteps"] == 0));
}
