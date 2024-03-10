use std::sync::Arc;

use actix_web::{get, web};
use tokio::sync::Mutex;

use crate::{spotify::models::SpotifyAuthResponse, twitch_auth::TwitchUserAuthResponse};

pub struct BotAuthState {
    pub twitch_auth_code: Arc<Mutex<String>>,
    pub spotify_auth_code: Arc<Mutex<String>>,
}

#[get("/auth")]
async fn auth(
    info: web::Query<TwitchUserAuthResponse>,
    auth_state: web::Data<BotAuthState>,
) -> String {
    let mut auth_token = auth_state.twitch_auth_code.lock().await;
    *auth_token = info.code.clone();

    tracing::info!("Received twitch token {}", info.code.clone());

    "You can close this now 🎉".into()
}

#[get("/spotify-auth")]
async fn spotify_auth(
    info: web::Query<SpotifyAuthResponse>,
    auth_state: web::Data<BotAuthState>,
) -> String {
    let mut auth_token = auth_state.spotify_auth_code.lock().await;
    *auth_token = info.code.clone();

    tracing::info!("Received spotify token {}", info.code.clone());

    "You can close this now 🎉".into()
}
