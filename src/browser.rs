pub async fn open_browser_and_authenticate_twitch(client_id: String, port: u16) {
    if tokio::fs::metadata("twitch_token.json").await.is_ok() {
        return;
    }

    let scopes = vec!["chat:edit", "chat:read"].join("+").replace(":", "%3A");

    let open_params = format!(
        "response_type=code&client_id={}&redirect_uri=http://localhost:{}/auth&scope={}",
        client_id, port, scopes
    );

    let _ = open::that(format!(
        "https://id.twitch.tv/oauth2/authorize?{}",
        open_params
    ));
}

pub async fn open_browser_and_authenticate_spotify(client_id: String, port: u16) {
    if tokio::fs::metadata("spotify_token.json").await.is_ok() {
        return;
    }

    let scopes = "user-modify-playback-state user-read-playback-state";

    let open_params = format!(
        "response_type=code&client_id={}&redirect_uri=http://localhost:{}/spotify-auth&scope={}",
        client_id, port, scopes
    );

    let _ = open::that(format!(
        "https://accounts.spotify.com/authorize?{}",
        open_params
    ));
}
