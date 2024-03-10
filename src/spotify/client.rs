use base64::Engine;
use chrono::Utc;
use tokio::io::AsyncReadExt;

use crate::error::TwitchBotResult;
use std::collections::HashMap;

use super::models::{SpotifySearchResult, SpotifyToken, SpotifyTrack, SpotifyTrackResults};

#[derive(Debug, Clone)]
pub struct SpotifyClient {
    pub client_id: String,
    pub client_secret: String,
    pub token: Option<SpotifyToken>,
}

impl SpotifyClient {
    pub async fn create_async(
        client_id: String,
        client_secret: String,
        auth_token: String,
        port: u16,
    ) -> TwitchBotResult<Self> {
        //Get token from file, if it doens't exist, make request
        if let Ok(mut token_file) = tokio::fs::File::open("spotify_token.json").await {
            let mut contents = String::new();
            token_file.read_to_string(&mut contents).await?;
            let token: SpotifyToken = serde_json::from_str(&contents)?;

            return Ok(SpotifyClient {
                client_id,
                client_secret,
                token: Some(token),
            });
        }

        let url = "https://accounts.spotify.com/api/token";
        let redirect_uri = format!("http://localhost:{}/spotify-auth", port);

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", &auth_token);
        params.insert("redirect_uri", &redirect_uri);

        let client = reqwest::Client::new();
        let base64auth = base64::engine::general_purpose::STANDARD
            .encode(&format!("{}:{}", client_id, client_secret));

        let request = client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Basic {}", base64auth))
            .form(&params)
            .send()
            .await?;

        let mut token = request.json::<SpotifyToken>().await?;
        token.created_at = Some(Utc::now());

        //Save access token to file
        tokio::fs::write("spotify_token.json", serde_json::to_string(&token)?).await?;

        Ok(SpotifyClient {
            client_id,
            client_secret,
            token: Some(token),
        })
    }

    pub async fn search_async(&self, query: &str) -> TwitchBotResult<SpotifyTrackResults> {
        let url = format!("https://api.spotify.com/v1/search?q={}&type=track", query);

        let client = reqwest::Client::new();

        let request = client
            .get(url)
            .bearer_auth(self.token.clone().unwrap().access_token)
            .send()
            .await?;

        let response = request.json::<SpotifySearchResult>().await?;

        Ok(response.tracks)
    }

    pub async fn queue_track(&self, track: &SpotifyTrack) -> TwitchBotResult<()> {
        let url = format!(
            "https://api.spotify.com/v1/me/player/queue?uri=spotify:track:{}",
            track.id
        );

        let client = reqwest::Client::new();

        let _response = client
            .post(url)
            .bearer_auth(self.token.clone().unwrap().access_token)
            .header("content-length", 0)
            .send()
            .await?;

        Ok(())
    }
}
