use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SpotifyAuthResponse {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpotifyToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifySearchResult {
    pub tracks: SpotifyTrackResults,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyTrackResults {
    pub total: u64,
    pub items: Vec<SpotifyTrack>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyArtist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub duration_ms: u64,
    pub artists: Vec<SpotifyArtist>,
}

impl fmt::Display for SpotifyTrack {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.name)?;
        fmt.write_str(" - ")?;
        fmt.write_str(&self.artists[0].name)?;
        Ok(())
    }
}

impl Default for SpotifyTrackResults {
    fn default() -> Self {
        Self {
            total: 0,
            items: Vec::new(),
        }
    }
}
