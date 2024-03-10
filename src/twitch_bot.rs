use std::{sync::Arc, time::Duration};

use tokio::{sync::Mutex, time::sleep};
use twitch_irc::{
    login::RefreshingLoginCredentials,
    message::ServerMessage,
    transport::tcp::{TCPTransport, TLS},
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{
    commands::get_command,
    error::TwitchBotResult,
    spotify::client::SpotifyClient,
    twitch_auth::{get_user_access_token_async, TwitchTokenStorage},
};

pub async fn run_async(
    client_id: String,
    client_secret: String,
    spotify_id: String,
    spotify_secret: String,
    port: u16,
    twitch_auth_token: Arc<Mutex<String>>,
    spotify_auth_token: Arc<Mutex<String>>,
) -> TwitchBotResult<()> {
    let twitch_token_exists = tokio::fs::metadata("twitch_token.json").await.is_ok();
    let spotify_token_exists = tokio::fs::metadata("spotify_token.json").await.is_ok();

    if !twitch_token_exists {
        while twitch_auth_token.lock().await.as_str() == "" {
            tracing::info!("Waiting for Twitch auth token");
            sleep(Duration::from_millis(1000)).await;
        }
    }

    if !spotify_token_exists {
        while spotify_auth_token.lock().await.as_str() == "" {
            tracing::info!("Waiting for Spotify auth token");
            sleep(Duration::from_millis(1000)).await;
        }
    }

    let spotify_auth_token_value = spotify_auth_token.lock().await.clone();
    let spotify_client =
        SpotifyClient::create_async(spotify_id, spotify_secret, spotify_auth_token_value, port)
            .await?;

    let twitch_auth_token_value = twitch_auth_token.lock().await.clone();
    let token = get_user_access_token_async(
        client_id.clone(),
        client_secret.clone(),
        twitch_auth_token_value,
        port,
    )
    .await?;

    let storage = TwitchTokenStorage { token };

    let credentials = RefreshingLoginCredentials::init(client_id, client_secret, storage);

    let twitch_config = ClientConfig::new_simple(credentials);
    let (mut incoming_messages, client) = TwitchIRCClient::<
        SecureTCPTransport,
        RefreshingLoginCredentials<TwitchTokenStorage>,
    >::new(twitch_config);

    client.join("vynny_".to_owned())?;

    while let Some(message) = incoming_messages.recv().await {
        process_message(&client, &spotify_client, message).await;
    }

    Ok(())
}

async fn process_message(
    client: &TwitchIRCClient<TCPTransport<TLS>, RefreshingLoginCredentials<TwitchTokenStorage>>,
    spotify_client: &SpotifyClient,
    message: ServerMessage,
) {
    match message {
        ServerMessage::Privmsg(msg) => {
            let user = msg.sender.name;
            let msg_text = msg.message_text;

            tracing::info!("{}: {}", user, msg_text);

            if let Some(response) = parse_command(msg_text, spotify_client).await {
                let _ = client.privmsg("vynny_".to_string(), response).await;
            }
        }
        _ => (),
    }
}

async fn parse_command(msg: String, spotify_client: &SpotifyClient) -> Option<String> {
    let command_message = msg.split_whitespace().nth(0)?.to_lowercase();
    let arguments_string: String = msg.split_whitespace().skip(1).collect();

    if command_message.starts_with("!") {
        if let Some(command) = get_command(command_message.to_string()) {
            let response = command.response;

            if let Some(api_call) = command.api_call {
                let api_response = match api_call.as_str() {
                    "play_track" => {
                        let track = spotify_client
                            .search_async(arguments_string.as_str())
                            .await
                            .unwrap_or_default()
                            .items
                            .get(0)?
                            .clone();

                        let _ = spotify_client.queue_track(&track).await;
                        response.replace("<song>", track.to_string().as_str())
                    }
                    _ => "".to_string(),
                };

                return Some(api_response);
            }

            return Some(response);
        }
    }

    None
}
