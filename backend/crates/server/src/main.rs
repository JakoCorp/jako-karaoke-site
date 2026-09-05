use std::sync::Arc;

use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use server::{config, routes, state::AppState, storage::FileStore, tasks};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".parse().unwrap()),
        )
        .init();

    let config = config::Config::from_env().expect("invalid configuration");
    let pool = db::connect(&config.database_url)
        .await
        .expect("failed to connect to database");
    let store = FileStore::new(&config.storage_path, &config.storage_base_url);

    let twitch_oauth = Arc::new(
        BasicClient::new(ClientId::new(config.twitch_client_id.clone()))
            .set_client_secret(ClientSecret::new(config.twitch_client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new("https://id.twitch.tv/oauth2/authorize".to_string())
                    .expect("valid Twitch auth URL"),
            )
            .set_token_uri(
                TokenUrl::new("https://id.twitch.tv/oauth2/token".to_string())
                    .expect("valid Twitch token URL"),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("{}/auth/twitch/callback", config.base_url))
                    .expect("valid redirect URL"),
            ),
    );
    let discord_oauth = Arc::new(
        BasicClient::new(ClientId::new(config.discord_client_id.clone()))
            .set_client_secret(ClientSecret::new(config.discord_client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new("https://discord.com/oauth2/authorize".to_string())
                    .expect("valid Discord auth URL"),
            )
            .set_token_uri(
                TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
                    .expect("valid Discord token URL"),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!("{}/auth/discord/callback", config.base_url))
                    .expect("valid redirect URL"),
            ),
    );

    let http_client = reqwest::Client::new();

    let state = AppState {
        pool,
        store,
        config: config.clone(),
        twitch_oauth,
        discord_oauth,
        http_client,
    };
    tasks::spawn_background_tasks(state.pool.clone());

    let app = routes::build_router(state);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}
