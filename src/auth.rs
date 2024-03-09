use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::Deserialize;
use tokio::sync::Mutex;
use twitch_irc::login::{TokenStorage, UserAccessToken};

use crate::error::TwitchBotResult;

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
pub struct TwitchTokenStorage {
    pub token: UserAccessToken,
}

#[async_trait]
impl TokenStorage for TwitchTokenStorage {
    type LoadError = crate::error::TwitchBotError;
    type UpdateError = crate::error::TwitchBotError;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        Ok(self.token.clone())
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        self.token = token.clone();
        Ok(())
    }
}

#[derive(Deserialize)]
struct TwitchOAuthResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
}

pub async fn get_user_access_token_async(
    client_id: String,
    client_secret: String,
    user_auth_code: String,
    port: u16,
) -> TwitchBotResult<UserAccessToken> {
    //Make http request for access token
    let client = reqwest::Client::new();
    let url = "https://id.twitch.tv/oauth2/token";
    let body = format!("client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri=http://localhost:{}/auth",
                       client_id, client_secret, user_auth_code, port);

    let auth_request = client.post(url).body(body).send().await?;

    let auth_response = auth_request.json::<TwitchOAuthResponse>().await?;

    Ok(UserAccessToken {
        access_token: auth_response.access_token,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::try_seconds(auth_response.expires_in).unwrap()),
        refresh_token: auth_response.refresh_token,
    })
}
