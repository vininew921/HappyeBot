use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::Deserialize;
use tokio::sync::Mutex;
use twitch_irc::login::{TokenStorage, UserAccessToken};

pub struct TwitchAuthState {
    pub auth_code: Arc<Mutex<String>>,
}

#[derive(Deserialize)]
pub struct TwitchUserAuthResponse {
    pub code: String,
    pub scope: String,
    pub state: Option<String>,
}

#[derive(Debug)]
pub struct HappyeBotTokenStorage {
    pub token: Option<UserAccessToken>,
}

#[async_trait]
impl TokenStorage for HappyeBotTokenStorage {
    type LoadError = std::io::Error;
    type UpdateError = std::io::Error;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        self.token.clone().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Token not found",
        ))
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        self.token = Some(token.clone());
        Ok(())
    }
}

#[derive(Deserialize)]
struct TwitchOAuthResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
}

pub async fn get_user_access_token(
    client_id: String,
    client_secret: String,
    user_auth_code: String,
    port: u16,
) -> UserAccessToken {
    //Make http request for access token
    let client = reqwest::Client::new();
    let url = "https://id.twitch.tv/oauth2/token";
    let body = format!("client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri=http://localhost:{}/auth", client_id, client_secret, user_auth_code, port);

    let auth_response = client
        .post(url)
        .body(body)
        .send()
        .await
        .expect("Could not post request for auth token")
        .json::<TwitchOAuthResponse>()
        .await
        .unwrap();

    UserAccessToken {
        access_token: auth_response.access_token,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::try_seconds(auth_response.expires_in).unwrap()),
        refresh_token: auth_response.refresh_token,
    }
}
