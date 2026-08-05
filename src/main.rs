use tokio::net::TcpListener;

use rust_friendsactivity_backend::{config::Config, db, features, router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env();

    let pool = db::connect(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    features::activity::weekly_reset::spawn(pool.clone());

    let app = router::build_router(pool);
    let listener = TcpListener::bind(config.bind_addr).await?;

    tracing::info!("Responding at http://{}", config.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
