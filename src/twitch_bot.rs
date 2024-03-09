use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{sync::Mutex, time::sleep};
use twitch_irc::{
    login::RefreshingLoginCredentials,
    message::ServerMessage,
    transport::tcp::{TCPTransport, TLS},
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{
    auth::{get_user_access_token_async, TwitchTokenStorage},
    commands::get_command,
    error::TwitchBotResult,
};

pub async fn run_async(
    client_id: String,
    client_secret: String,
    port: u16,
    auth_token: Arc<Mutex<String>>,
    shutdown_marker: Arc<AtomicBool>,
) -> TwitchBotResult<()> {
    while auth_token.lock().await.as_str() == "" {
        if shutdown_marker.load(Ordering::SeqCst) {
            return Ok(());
        }

        tracing::info!("Waiting for auth token");
        sleep(Duration::from_millis(1000)).await;
    }

    let auth_token_value = auth_token.lock().await.clone();
    let token = get_user_access_token_async(
        client_id.clone(),
        client_secret.clone(),
        auth_token_value,
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
        if shutdown_marker.load(Ordering::SeqCst) {
            break;
        }

        process_message(&client, message).await;
    }

    Ok(())
}

async fn process_message(
    client: &TwitchIRCClient<TCPTransport<TLS>, RefreshingLoginCredentials<TwitchTokenStorage>>,
    message: ServerMessage,
) {
    match message {
        ServerMessage::Privmsg(msg) => {
            let user = msg.sender.name;
            let msg_text = msg.message_text;

            tracing::info!("{}: {}", user, msg_text);

            if let Some(response) = parse_command(msg_text, user) {
                let _ = client.privmsg("vynny_".to_string(), response).await;
            }
        }
        _ => (),
    }
}

fn parse_command(msg: String, user: String) -> Option<String> {
    let command_message = msg.split_whitespace().nth(0)?.to_lowercase();

    if command_message.starts_with("!") {
        if let Some(command) = get_command(command_message.to_string()) {
            tracing::info!("COMMAND: {}", command_message);

            let response = command.response;
            if command.tag_user {
                return Some(response.replace("<user>", user.as_str()));
            }

            return Some(response);
        }
    }

    None
}
